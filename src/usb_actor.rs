use std::collections::HashMap;

use log::{debug, info, warn};
use smart_leds::RGB;

use crate::{barrier::Actuator, keycodes::synergy_to_hid, utils::set_led};

extern "C" {
    fn usb_util_init();
    fn usb_util_key_down(key: u8, button: u16);
    fn usb_util_key_up(button: u16);
    fn usb_util_move_to_pos(x: u16, y: u16);
    fn usb_util_mouse_button(button: u8);
    fn usb_util_mouse_button_up(button: u8);
    fn usb_util_mouse_wheel(scroll: i16, pan: i16);
    fn usb_util_reset_key_states();
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
}

impl UsbHidActuator {
    pub fn new(width: u16, height: u16) -> Self {
        unsafe { usb_util_init() }
        Self {
            width,
            height,
            x: 0,
            y: 0,
            options: HashMap::new(),
            flip_mouse_wheel: env!("REVERSED_WHEEL").parse().unwrap_or(0),
            v_scroll_scale: env!("V_SCROLL_SCALE").parse().unwrap_or(1.0),
            h_scroll_scale: env!("H_SCROLL_SCALE").parse().unwrap_or(1.0),
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
        (self.x, self.y)
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
        debug!("Set cursor position to {x} {y}");
        unsafe { usb_util_move_to_pos(self.x, self.y) }
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        self.x = (self.x as i32 + x as i32) as u16;
        self.y = (self.y as i32 + y as i32) as u16;
        debug!("Move cursor by {x} {y}, now at {} {}", self.x, self.y);
        unsafe { usb_util_move_to_pos(self.x, self.y) }
    }

    fn mouse_down(&mut self, button: i8) {
        debug!("Mouse down {button}");
        unsafe { usb_util_mouse_button(button as u8) }
    }

    fn mouse_up(&mut self, button: i8) {
        debug!("Mouse up {button}");
        unsafe { usb_util_mouse_button_up(button as u8) }
    }

    fn mouse_wheel(&mut self, x: i16, y: i16) {
        debug!(
            "Mouse wheel {x}*{} {y}*{}",
            self.h_scroll_scale, self.v_scroll_scale
        );
        let x = (x as f32 * self.h_scroll_scale / 120.0) as i16;
        let y = (y as f32 * self.v_scroll_scale / 120.0) as i16;
        debug!("Mouse wheel {x} {y}");
        unsafe { usb_util_mouse_wheel(if self.flip_mouse_wheel > 0 { -y } else { y }, x) }
    }

    fn key_down(&mut self, key: u16, mask: u16, button: u16) {
        debug!("Key down {key} {mask} {button}");
        let hid = synergy_to_hid(key);
        if hid == 0 {
            warn!("Keycode not found");
            return;
        }
        debug!("Keycode: {}", hid);
        unsafe { usb_util_key_down(hid, button) }
    }

    fn key_repeat(&mut self, key: u16, mask: u16, button: u16, count: u16) {
        debug!("Key repeat {key} {mask} {button} {count}");
        let hid = synergy_to_hid(key);
        debug!("Keycode: {}", hid);
        if hid == 0 {
            warn!("Keycode not found");
            return;
        }
        for _ in 0..count {
            unsafe { usb_util_key_down(hid, button) }
        }
    }

    fn key_up(&mut self, key: u16, mask: u16, button: u16) {
        debug!("Key up {key} {mask} {button}");
        let hid = synergy_to_hid(key);
        debug!("Keycode: {}", hid);
        if hid == 0 {
            warn!("Keycode not found");
            return;
        }
        unsafe { usb_util_key_up(button) }
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
        unsafe { usb_util_reset_key_states() }
    }

    fn leave(&mut self) {
        info!("Leave");
        // Dim yellow
        set_led(RGB { r: 40, g: 20, b: 0 });
        unsafe { usb_util_reset_key_states() }
    }
}
