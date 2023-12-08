#[allow(dead_code)]
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
pub use neopixel_status::set_status;

#[cfg(feature = "m5atoms3")]
pub mod lcd_status;

#[cfg(feature = "m5atoms3")]
pub use lcd_status::set_status;

// For boards without status display
#[cfg(not(any(feature = "m5atoms3", feature = "m5atoms3lite")))]
pub mod dummy_status;
#[cfg(not(any(feature = "m5atoms3", feature = "m5atoms3lite")))]
pub use dummy_status::set_status;
