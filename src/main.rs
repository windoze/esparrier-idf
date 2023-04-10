use std::sync::Mutex;

use anyhow::Result;
use const_env::from_env;
use esp_idf_hal::{gpio::AnyInputPin, prelude::Peripherals};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use lazy_static::lazy_static;
use log::{error, info};

mod barrier;
mod keycodes;
mod paste_button;
mod reports;
mod settings;
mod usb_actor;
mod utils;
mod status;

use paste_button::start_paste_button_task;
use settings::*;
use utils::*;

use crate::{barrier::ThreadedActuator, usb_actor::UsbHidActuator, status::{set_status, Status}};

#[from_env("DEBUG_INIT_USB")]
pub const INIT_USB: bool = true;

// M5Atom S3 and Lite has a button on GPIO 41
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

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    set_status(Status::Start);

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // Initialize WIFI
    let _wifi = wifi(peripherals.modem, sysloop).unwrap();

    // Blue when connected to wifi
    set_status(Status::WifiConnected);

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
    set_status(Status::WifiConnected) ;// set_led(RGB { r: 0, g: 0, b: 255 });

    panic!("Disconnected, restarting...")
}
