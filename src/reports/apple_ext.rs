use super::usb_util_apple_ext_report;

pub struct AppleExtReport {
    pub code: u8,
}

impl AppleExtReport {
    pub fn new() -> Self {
        Self { code: 0 }
    }

    pub fn press(&mut self, code: u8) {
        self.code = code;
        self.send();
    }

    pub fn release(&mut self) {
        self.code = 0;
        self.send();
    }

    pub fn clear(&mut self) {
        self.code = 0;
        self.send();
    }

    fn send(&self) {
        unsafe {
            usb_util_apple_ext_report(self.code);
        }
    }
}