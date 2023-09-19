use std::sync::Mutex;

use super::Status;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, RgbColor},
};
use esp_idf_hal::{gpio, spi};
use lazy_static::lazy_static;

pub type DisplayType = mipidsi::Display<
    SPIInterfaceNoCS<
        spi::SpiDeviceDriver<'static, spi::SpiDriver<'static>>,
        gpio::PinDriver<'static, gpio::Gpio33, gpio::Output>,
    >,
    mipidsi::models::ST7789,
    gpio::PinDriver<'static, gpio::Gpio34, gpio::Output>,
>;

pub struct LCDStatusDisplay {
    display: DisplayType,
}

impl LCDStatusDisplay {
    fn new(display: DisplayType) -> Self {
        Self { display }
    }

    pub fn set_status(&mut self, status: Status) {
        match status {
            Status::None => {
                self.display.clear(Rgb565::BLACK).unwrap();
            }
            Status::Start => {
                self.display.clear(Rgb565::RED).unwrap();
            }
            Status::WifiConnected => {
                self.display.clear(Rgb565::BLUE).unwrap();
            }
            Status::Activated => {
                self.display.clear(Rgb565::GREEN).unwrap();
            }
            Status::Deactivated => {
                self.display.clear(Rgb565::YELLOW).unwrap();
            }
            Status::ClipboardSize(_) => {
                // self.clipboard_size.store(size, Ordering::Relaxed);
            }
        }
    }
}

lazy_static! {
    static ref DISPLAY: Mutex<Option<LCDStatusDisplay>> = Mutex::new(None);
}

pub fn init_display(display: DisplayType) {
    let mut display = LCDStatusDisplay::new(display);
    display.set_status(Status::Start);
    *DISPLAY.lock().unwrap() = Some(display);
}

pub fn set_status(status: Status) {
    if let Some(display) = DISPLAY.lock().unwrap().as_mut() {
        display.set_status(status)
    }
}
