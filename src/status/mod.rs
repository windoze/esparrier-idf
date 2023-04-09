#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    None,
    Start,
    WifiConnected,
    Activated,
    Deactivated,
    ClipboardSize(usize),
}

pub trait StatusDisplay {
    fn set_status(&self, status: Status);
    fn need_attention(&self, code: u8);
}

#[cfg(feature = "m5atoms3lite")]
mod neopixel_status;

#[cfg(feature = "m5atoms3lite")]
type StatusDisplayType = neopixel_status::NeoPixelStatus;

#[cfg(feature = "m5atoms3")]
mod lcd_status;

#[cfg(feature = "m5atoms3")]
type StatusDisplayType = lcd_status::LcdStatus;

lazy_static :: lazy_static! {
    static ref STATUS_DISPLAY: StatusDisplayType = StatusDisplayType::new();
}

pub fn set_status(status: Status) {
    STATUS_DISPLAY.set_status(status);
}
