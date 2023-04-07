use log::warn;

mod abs_mouse;
mod consumer_control;
mod keyboard;

extern "C" {
    fn usb_util_init();
    fn usb_util_keyboard_report(modifier: u8, keycode: *const u8);
    fn usb_util_abs_mouse_report(buttons: u8, x: u16, y: u16, wheel: i8, pan: i8);
    fn usb_util_consumer_report(code: u16);
}

use abs_mouse::AbsMouseReport;
use consumer_control::ConsumerControlReport;
use keyboard::KeyboardReport;

use crate::keycodes::KeyCode;

pub enum HidReportType {
    KeyPress { key_code: KeyCode },
    KeyRelease { key_code: KeyCode },
    MouseMove { x: u16, y: u16 },
    MouseMoveRelative { x: i16, y: i16 },
    MouseDown { button: u8 },
    MouseUp { button: u8 },
    MouseWheel { scroll: i8, pan: i8 },
}

pub struct HidReport {
    mouse: AbsMouseReport,
    keyboard: KeyboardReport<6>,
    consumer_control: ConsumerControlReport,
}

impl HidReport {
    pub fn new() -> Self {
        Self {
            mouse: AbsMouseReport::new(),
            keyboard: KeyboardReport::new(),
            consumer_control: ConsumerControlReport::new(),
        }
    }

    pub fn init(&mut self) {
        if crate::INIT_USB {
            unsafe { usb_util_init() }
        } else {
            warn!("Skipping USB init")
        }
    }

    pub fn get_mouse_position(&self) -> (u16, u16) {
        self.mouse.get_position()
    }

    pub fn send(&mut self, report: HidReportType) {
        match report {
            HidReportType::KeyPress { key_code } => match key_code {
                KeyCode::None => (),
                KeyCode::Consumer(code) => self.consumer_control.press(code),
                KeyCode::Key(hid_key) => self.keyboard.press(hid_key),
            },
            HidReportType::KeyRelease { key_code } => match key_code {
                KeyCode::None => (),
                KeyCode::Consumer(_) => self.consumer_control.release(),
                KeyCode::Key(hid_key) => self.keyboard.release(hid_key),
            },
            HidReportType::MouseMove { x, y } => self.mouse.move_to(x, y),
            HidReportType::MouseMoveRelative { x, y } => self.mouse.move_by(x, y),
            HidReportType::MouseDown { button } => self.mouse.mouse_down(button),
            HidReportType::MouseUp { button } => self.mouse.mouse_up(button),
            HidReportType::MouseWheel { scroll, pan } => self.mouse.mouse_wheel(scroll, pan),
        }
    }

    pub fn clear(&mut self) {
        self.mouse.clear();
        self.keyboard.clear();
        self.consumer_control.clear();
    }
}
