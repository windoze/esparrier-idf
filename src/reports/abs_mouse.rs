use super::usb_util_abs_mouse_report;

pub struct AbsMouseReport {
    button: u8,
    x: u16,
    y: u16,
}

impl AbsMouseReport {
    pub fn new() -> Self {
        Self {
            button: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
        self.send(None, None);
    }

    pub fn move_by(&mut self, x: i16, y: i16) {
        self.x = self.x.wrapping_add(x as u16);
        self.y = self.y.wrapping_add(y as u16);
        self.send(None, None);
    }

    pub fn mouse_down(&mut self, button: u8) {
        self.button |= button;
        self.send(None, None);
    }

    pub fn mouse_up(&mut self, button: u8) {
        self.button &= !button;
        self.send(None, None);
    }

    pub fn mouse_wheel(&mut self, scroll: i8, pan: i8) {
        self.send(scroll, pan);
    }

    pub fn clear(&mut self) {
        self.button = 0;
        // NOTE: Preserve the last position
        // self.x = 0;
        // self.y = 0;
        self.send(None, None);
    }

    pub fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    fn send<S: Into<Option<i8>>, P: Into<Option<i8>>>(&self, scroll: S, pan: P) {
        // Scale the position to the screen size
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

