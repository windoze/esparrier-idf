use std::{
    sync::{
        mpsc::{Receiver, SyncSender},
        Mutex,
    },
    time::{Duration, Instant},
};

use const_env::from_env;
use lazy_static::lazy_static;
use log::debug;
use smart_leds::{SmartLedsWrite, RGB};
use ws2812_esp32_rmt_driver::{driver::color::LedPixelColorImpl, LedPixelEsp32Rmt, Ws2812Esp32Rmt};

// M5Atom S3 Lite has a status NeoPixel on GPIO 35
#[from_env("STATUS_LED_PIN")]
const STATUS_LED_PIN: u32 = 35;

const FLASH_INTERVAL: Duration = Duration::from_millis(100);
const TIMEOUT: Duration = Duration::from_millis(50);

use super::{Status, StatusDisplay};

lazy_static! {
    static ref STATUS_LED: Mutex<LedPixelEsp32Rmt<RGB<u8>, LedPixelColorImpl<3, 1, 0, 2, 255>>> =
        Mutex::new(Ws2812Esp32Rmt::new(0, STATUS_LED_PIN).unwrap());
}

struct NeoPixelStatusTask {
    rx: Receiver<Status>,
}

impl NeoPixelStatusTask {
    fn new(rx: Receiver<Status>) -> Self {
        Self { rx }
    }

    fn set_color(&self, color: RGB<u8>) {
        STATUS_LED
            .lock()
            .map(|mut led| led.write(std::iter::once(color)))
            .ok();
    }

    fn set_status(&self, status: Status) {
        match status {
            Status::None => self.set_color(RGB::new(0, 0, 0)),
            Status::Start => self.set_color(RGB::new(128, 0, 0)),
            Status::WifiConnected => self.set_color(RGB::new(0, 0, 128)),
            Status::Activated => self.set_color(RGB::new(0, 64, 0)),
            Status::Deactivated => self.set_color(RGB::new(40, 24, 0)),
            Status::ClipboardSize(_) => {}
        }
    }

    fn run(&self) {
        let mut current_status = Status::None;
        let mut flash_start: Option<Instant> = None;
        let mut led_on = true;
        loop {
            match self.rx.recv_timeout(TIMEOUT) {
                Ok(status) => {
                    if status != current_status {
                        match status {
                            Status::Start | Status::WifiConnected => {
                                debug!("Start flashing LED");
                                self.set_status(status);
                                flash_start = Some(Instant::now());
                                led_on = true;
                            }
                            _ => {
                                flash_start = None;
                                led_on = true;
                            }
                        }
                    }
                    current_status = status;
                    self.set_status(current_status);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(start) = flash_start {
                        if start.elapsed() > FLASH_INTERVAL {
                            debug!("Flashing LED");
                            flash_start = Some(Instant::now());
                            led_on = !led_on;
                            if led_on {
                                self.set_status(current_status);
                            } else {
                                self.set_color(RGB::new(0, 0, 0));
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}

pub struct NeoPixelStatus {
    tx: SyncSender<Status>,
}

impl NeoPixelStatus {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let task = NeoPixelStatusTask::new(rx);
            task.run();
        });
        Self { tx }
    }
}

impl StatusDisplay for NeoPixelStatus {
    fn set_status(&self, status: Status) {
        self.tx.send(status).ok();
    }

    fn need_attention(&self, _code: u8) {
        // do nothing
    }
}

lazy_static :: lazy_static! {
    static ref STATUS_DISPLAY: NeoPixelStatus = NeoPixelStatus::new();
}

pub fn set_status(status: Status) {
    STATUS_DISPLAY.set_status(status);
}
