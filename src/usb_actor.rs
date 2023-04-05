use std::collections::HashMap;

use log::{debug, info, warn};
use smart_leds::RGB;

use crate::{
    barrier::Actuator,
    keycodes::{synergy_mouse_button, synergy_to_hid},
    utils::set_led,
};

const INIT_USB: bool = true;

extern "C" {
    fn usb_util_init();
    fn usb_util_keyboard_report(modifier: u8, keycode: *const u8);
    fn usb_util_abs_mouse_report(buttons: u8, x: u16, y: u16, wheel: i8, pan: i8);
}

struct AbsMouseReport {
    button: u8,
    x: u16,
    y: u16,
}

impl AbsMouseReport {
    fn new() -> Self {
        Self {
            button: 0,
            x: 0,
            y: 0,
        }
    }

    fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
        self.send(None, None);
    }

    fn move_by(&mut self, x: i16, y: i16) {
        self.x = self.x.wrapping_add(x as u16);
        self.y = self.y.wrapping_add(y as u16);
        self.send(None, None);
    }

    fn mouse_down(&mut self, button: u8) {
        self.button |= button;
        self.send(None, None);
    }

    fn mouse_up(&mut self, button: u8) {
        self.button &= !button;
        self.send(None, None);
    }

    fn mouse_wheel(&mut self, scroll: i8, pan: i8) {
        self.send(scroll, pan);
    }

    fn clear(&mut self) {
        self.button = 0;
        // NOTE: Preserve the last position
        // self.x = 0;
        // self.y = 0;
        self.send(None, None);
    }

    fn send<S: Into<Option<i8>>, P: Into<Option<i8>>>(&self, scroll: S, pan: P) {
        unsafe {
            usb_util_abs_mouse_report(
                self.button,
                self.x,
                self.y,
                scroll.into().unwrap_or_default(),
                pan.into().unwrap_or_default(),
            );
        }
    }
}

struct KeyReport<const N: usize> {
    modifier: u8,
    keycode: [u8; N],
}

impl<const N: usize> KeyReport<N> {
    fn new() -> Self {
        Self {
            modifier: 0,
            keycode: [0; N],
        }
    }

    fn press_key(&mut self, key: u8) {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier |= modifier,
            None => {
                let mut found = false;
                for i in 0..N {
                    if self.keycode[i] == 0 {
                        self.keycode[i] = key;
                        found = true;
                        break;
                    }
                }
                if !found {
                    // roll over the first key
                    for i in 1..N {
                        self.keycode.swap(i - 1, i);
                    }
                    self.keycode[N - 1] = key;
                }
            }
        }
        self.send();
    }

    fn release_key(&mut self, key: u8) {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier &= !modifier,
            None => {
                for i in 0..N {
                    if self.keycode[i] == key {
                        self.keycode[i] = 0;
                        break;
                    }
                }
                // Compact the keycode array
                let mut pos = 0;
                for i in 0..N {
                    if self.keycode[i] != 0 {
                        self.keycode.swap(i, pos);
                        pos += 1;
                    }
                }
            }
        }
        self.send();
    }

    fn clear(&mut self) {
        self.modifier = 0;
        self.keycode = [0; N];
        self.send();
    }

    fn send(&self) {
        unsafe {
            usb_util_keyboard_report(self.modifier, self.keycode.as_ptr());
        }
    }

    fn get_modifier(&self, key: u8) -> Option<u8> {
        match key {
            0xE0 => Some(0x01), // Left Control
            0xE1 => Some(0x02), // Left Shift
            0xE2 => Some(0x04), // Left Alt
            0xE3 => Some(0x08), // Left GUI
            0xE4 => Some(0x10), // Right Control
            0xE5 => Some(0x20), // Right Shift
            0xE6 => Some(0x40), // Right Alt
            0xE7 => Some(0x80), // Right GUI
            _ => None,
        }
    }
}

pub struct UsbHidActuator {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub options: HashMap<String, u32>,
    pub flip_mouse_wheel: u8,
    pub v_scroll_scale: f32,
    pub h_scroll_scale: f32,

    mouse_report: AbsMouseReport,
    key_report: KeyReport<6>,
}

impl UsbHidActuator {
    pub fn new(width: u16, height: u16) -> Self {
        if INIT_USB {
            unsafe {
                usb_util_init();
            }
        }
        Self {
            width,
            height,
            x: 0,
            y: 0,
            options: HashMap::new(),
            flip_mouse_wheel: env!("REVERSED_WHEEL").parse().unwrap_or(0),
            v_scroll_scale: env!("V_SCROLL_SCALE").parse().unwrap_or(1.0),
            h_scroll_scale: env!("H_SCROLL_SCALE").parse().unwrap_or(1.0),
            mouse_report: AbsMouseReport::new(),
            key_report: KeyReport::new(),
        }
    }
}

impl Actuator for UsbHidActuator {
    fn connected(&mut self) {
        info!("Connected");
        // Dim yellow
        set_led(RGB { r: 40, g: 20, b: 0 });
    }

    fn disconnected(&mut self) {
        info!("Disconnected");
    }

    fn get_screen_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_cursor_position(&self) -> (u16, u16) {
        (self.mouse_report.x, self.mouse_report.y)
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) {
        debug!("Set cursor position to {x} {y}");
        self.mouse_report.move_to(x, y);
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        debug!("Move cursor by {x} {y}");
        self.mouse_report.move_by(x, y);
    }

    fn mouse_down(&mut self, button: i8) {
        debug!("Mouse down {button}");
        self.mouse_report.mouse_down(synergy_mouse_button(button));
    }

    fn mouse_up(&mut self, button: i8) {
        debug!("Mouse up {button}");
        self.mouse_report.mouse_up(synergy_mouse_button(button));
    }

    fn mouse_wheel(&mut self, x: i16, y: i16) {
        debug!(
            "Mouse wheel {x}*{} {y}*{}",
            self.h_scroll_scale, self.v_scroll_scale
        );
        let x = (x as f32 * self.h_scroll_scale / 120.0) as i16;
        let y = (y as f32 * self.v_scroll_scale / 120.0) as i16;
        debug!("Mouse wheel {x} {y}");
        self.mouse_report.mouse_wheel(y as i8, x as i8);
    }

    fn key_down(&mut self, key: u16, mask: u16, button: u16) {
        debug!("Key down {key} {mask} {button}");
        let hid = synergy_to_hid(key);
        if hid == 0 {
            warn!("Keycode not found");
            return;
        }
        debug!("Keycode: {}", hid);
        self.key_report.press_key(hid);
    }

    fn key_repeat(&mut self, key: u16, mask: u16, button: u16, count: u16) {
        // Looks we should ignore this as USB HID doesn't need to repeat key press
        debug!("Key repeat {key} {mask} {button} {count}");
    }

    fn key_up(&mut self, key: u16, mask: u16, button: u16) {
        debug!("Key up {key} {mask} {button}");
        let hid = synergy_to_hid(key);
        debug!("Keycode: {}", hid);
        if hid == 0 {
            warn!("Keycode not found");
            return;
        }
        self.key_report.release_key(hid);
    }

    fn set_options(&mut self, opts: std::collections::HashMap<String, u32>) {
        self.options = opts;
        info!("Set options {:#?}", self.options)
    }

    fn reset_options(&mut self) {
        self.options.clear();
        info!("Reset options")
    }

    fn enter(&mut self) {
        info!("Enter");
        // Lighter green
        set_led(RGB { r: 0, g: 64, b: 0 });
        self.key_report.clear();
        self.mouse_report.clear();
    }

    fn leave(&mut self) {
        info!("Leave");
        // Dim yellow
        set_led(RGB { r: 40, g: 20, b: 0 });
        self.key_report.clear();
        self.mouse_report.clear();
    }
}
