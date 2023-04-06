use anyhow::Result;
use const_env::from_env;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use log::{error, info};
use smart_leds::RGB;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

mod barrier;
mod usb_actor;
mod utils;
mod keycodes;
mod reports;

use utils::*;

use crate::usb_actor::UsbHidActuator;

pub const INIT_USB: bool = true;

// Constants from env
#[from_env("SCREEN_WIDTH")]
const SCREEN_WIDTH: u16 = 1920;
#[from_env("SCREEN_HEIGHT")]
const SCREEN_HEIGHT: u16 = 1080;

#[from_env("BARRIER_SERVER")]
const BARRIER_SERVER: &'static str = "127.0.0.1";
#[from_env("BARRIER_PORT")]
const BARRIER_PORT: u16 = 24800;
#[from_env("SCREEN_NAME")]
const SCREEN_NAME: &'static str = "ESPARRIER";

// M5Atom S3 Lite has a status NeoPixel on GPIO 35
#[from_env("STATUS_LED_PIN")]
const STATUS_LED_PIN: u32 = 35;

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Required before starting the wifi
    unsafe {
        nvs_flash_init();
    }

    *STATUS_LED.lock().unwrap() = Some(Ws2812Esp32Rmt::new(0, STATUS_LED_PIN).unwrap());

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    // Red on start
    set_led(RGB { r: 255, g: 0, b: 0 });

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // Initialize WIFI
    let _wifi = wifi(peripherals.modem, sysloop).unwrap();

    // Blue when connected to wifi
    set_led(RGB { r: 0, g: 0, b: 255 });

    let mut actor = UsbHidActuator::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    info!("Connecting to barrier...");
    match barrier::start(BARRIER_SERVER, BARRIER_PORT, SCREEN_NAME, &mut actor) {
        Ok(_) => {
            error!("Connection closed");
        }
        Err(e) => {
            error!("Connection failed: {}", e);
        }
    }
    set_led(RGB { r: 0, g: 0, b: 255 });

    panic!("Disconnected, restarting...")
}
