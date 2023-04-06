use super::usb_util_consumer_report;

pub struct ConsumerControlReport {
    pub code: u16,
}

impl ConsumerControlReport {
    pub fn new() -> Self {
        Self { code: 0 }
    }

    pub fn press(&mut self, code: u16) {
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
            usb_util_consumer_report(self.code);
        }
    }
}