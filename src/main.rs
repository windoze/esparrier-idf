use std::sync::Mutex;

use anyhow::Result;
use const_env::from_env;
use esp_idf_hal::{gpio::AnyInputPin, prelude::Peripherals};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use lazy_static::lazy_static;
use log::{error, info};
use smart_leds::RGB;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

mod barrier;
mod keycodes;
mod paste_button;
mod reports;
mod settings;
mod usb_actor;
mod utils;

use paste_button::start_paste_button_task;
use settings::*;
use utils::*;

use crate::{barrier::ThreadedActuator, usb_actor::UsbHidActuator};

#[from_env("DEBUG_INIT_USB")]
pub const INIT_USB: bool = true;

// M5Atom S3 Lite has a status NeoPixel on GPIO 35
#[from_env("STATUS_LED_PIN")]
const STATUS_LED_PIN: u32 = 35;

// M5Atom S3 Lite has a button on GPIO 41
#[from_env("PASTE_BUTTON_PIN")]
const PASTE_BUTTON_PIN: i32 = 41;

lazy_static! {
    static ref CLIPBOARD: Mutex<Vec<u8>> = Mutex::new(vec![]);
}

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

    let screen_width = get_screen_width();
    let screen_height = get_screen_height();
    let actor = UsbHidActuator::new(screen_width, screen_height);
    let mut actor = ThreadedActuator::new(screen_width, screen_height, actor);

    start_paste_button_task(
        unsafe { AnyInputPin::new(PASTE_BUTTON_PIN) },
        actor.get_sender(),
    );

    info!("Connecting to barrier...");
    match barrier::start(
        get_barrier_server(),
        get_barrier_port(),
        get_screen_name(),
        &mut actor,
    ) {
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
