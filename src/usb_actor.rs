use crate::{
    settings::*,
    status::{set_status, Status},
    CLIPBOARD,
};
use log::{debug, info, warn};

use crate::{
    barrier::Actuator,
    keycodes::{synergy_mouse_button, synergy_to_hid, KeyCode},
    reports::{HidReport, HidReportType},
    INIT_USB,
};

pub struct UsbHidActuator {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub flip_mouse_wheel: bool,
    pub v_scroll_scale: f32,
    pub h_scroll_scale: f32,

    hid_report: HidReport,
    server_buttons: [u16; 512],
}

impl UsbHidActuator {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            flip_mouse_wheel: get_reversed_wheel(),
            v_scroll_scale: get_v_scroll_scale(),
            h_scroll_scale: get_h_scroll_scale(),
            hid_report: HidReport::new(),
            server_buttons: [0; 512],
        }
    }

    fn clear(&mut self) {
        info!("Clear");
        self.hid_report.clear();
        self.server_buttons.fill(0);
    }
}

impl Actuator for UsbHidActuator {
    fn connected(&mut self) {
        info!("Connected");
        // Dim yellow
        set_status(Status::Deactivated);
        // Delay USB init until we're connected, make the code easier to debug
        self.hid_report.init();
    }

    fn disconnected(&mut self) {
        info!("Disconnected");
    }

    fn get_screen_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_cursor_position(&self) -> (u16, u16) {
        self.hid_report.get_mouse_position()
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) {
        debug!("Set cursor position to {x} {y}");
        self.hid_report.send(HidReportType::MouseMove { x, y });
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        debug!("Move cursor by {x} {y}");
        self.hid_report
            .send(HidReportType::MouseMoveRelative { x, y });
    }

    fn mouse_down(&mut self, button: i8) {
        debug!("Mouse down {button}");
        self.hid_report.send(HidReportType::MouseDown {
            button: synergy_mouse_button(button),
        });
    }

    fn mouse_up(&mut self, button: i8) {
        debug!("Mouse up {button}");
        self.hid_report.send(HidReportType::MouseUp {
            button: synergy_mouse_button(button),
        });
    }

    fn mouse_wheel(&mut self, x: i16, y: i16) {
        debug!(
            "Mouse wheel {x}*{} {y}*{}",
            self.h_scroll_scale, self.v_scroll_scale
        );
        let x = (x as f32 * self.h_scroll_scale / 120.0) as i16;
        let y = (y as f32 * self.v_scroll_scale / 120.0) as i16;
        debug!("Mouse wheel {x} {y}");
        if self.flip_mouse_wheel {
            self.hid_report.send(HidReportType::MouseWheel {
                scroll: -y as i8,
                pan: -x as i8,
            });
        } else {
            self.hid_report.send(HidReportType::MouseWheel {
                scroll: y as i8,
                pan: x as i8,
            });
        }
    }

    fn key_down(&mut self, key: u16, mask: u16, button: u16) {
        debug!("Key down {key} {mask} {button}");
        self.server_buttons[button as usize] = key;
        let hid = synergy_to_hid(key);
        if INIT_USB {
            debug!("Key Down {:#04x} -> Keycode: {:?}", key, hid);
        } else {
            info!("Key Down {:#04x} -> Keycode: {:?}", key, hid);
        }
        if matches!(hid, KeyCode::None) {
            warn!("Keycode not found");
            return;
        }
        self.hid_report
            .send(HidReportType::KeyPress { key_code: hid });
    }

    fn key_repeat(&mut self, key: u16, mask: u16, button: u16, count: u16) {
        // Looks we should ignore this as USB HID doesn't need to repeat key press
        debug!("Key repeat {key} {mask} {button} {count}");
    }

    fn key_up(&mut self, _key: u16, mask: u16, button: u16) {
        debug!("Key up {_key} {mask} {button}");
        let key = self.server_buttons[button as usize];
        if self.server_buttons[button as usize] != 0 {
            debug!("Key {key} up");
            self.server_buttons[button as usize] = 0;
        } else if key == 0 {
            info!("Key 0 up, clear all key down");
            self.hid_report.clear();
        } else {
            warn!("Key {key} up with no key down");
        }
        let hid = synergy_to_hid(key);
        if INIT_USB {
            debug!("Key Up {:#04x} -> Keycode: {:?}", key, hid);
        } else {
            info!("Key Up {:#04x} -> Keycode: {:?}", key, hid);
        }
        if matches!(hid, KeyCode::None) {
            warn!("Keycode not found");
            return;
        }
        self.hid_report
            .send(HidReportType::KeyRelease { key_code: hid });
    }

    fn set_clipboard(&mut self, mut data: Vec<u8>) {
        set_status(Status::ClipboardSize(data.len()));
        std::mem::swap(CLIPBOARD.lock().unwrap().as_mut(), &mut data);
    }

    fn set_options(&mut self, heartbeat: u32) {
        info!("Set heartbeat {heartbeat}")
    }

    fn reset_options(&mut self) {
        info!("Reset options")
    }

    fn enter(&mut self) {
        info!("Enter");
        // Lighter green
        set_status(Status::Activated);
        self.clear();
    }

    fn leave(&mut self) {
        info!("Leave");
        // Dim yellow
        set_status(Status::Deactivated);
        self.clear();
    }

    fn hid_key_down(&mut self, key: u8) {
        self.hid_report.send(HidReportType::KeyPress {
            key_code: KeyCode::Key(key),
        });
    }

    fn hid_key_up(&mut self, key: u8) {
        self.hid_report.send(HidReportType::KeyRelease {
            key_code: KeyCode::Key(key),
        });
    }
}
