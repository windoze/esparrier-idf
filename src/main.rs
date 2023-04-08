use std::sync::mpsc::SyncSender;

use anyhow::Result;
use barrier::ActMsg;
use const_env::from_env;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use log::{error, info};
use smart_leds::RGB;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

mod barrier;
mod button;
mod keycodes;
mod reports;
mod settings;
mod usb_actor;
mod utils;

use button::{start_button_task, ButtonCallback, ButtonState};
use settings::*;
use utils::*;

use crate::{barrier::ThreadedActuator, usb_actor::UsbHidActuator};

pub const INIT_USB: bool = false;

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

    let screen_width = get_screen_width();
    let screen_height = get_screen_height();
    let actor = UsbHidActuator::new(screen_width, screen_height);
    let mut actor = ThreadedActuator::new(screen_width, screen_height, actor);

    let b = TestButton {
        tx: actor.get_sender(),
    };

    start_button_task(peripherals.pins.gpio41.into(), b);

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

struct TestButton {
    tx: SyncSender<ActMsg>,
}

impl ButtonCallback for TestButton {
    fn on_button_event(&mut self, state: ButtonState) {
        match state {
            ButtonState::Up => {}
            ButtonState::Down => {
                if state == ButtonState::Down {
                    self.tx.send(ActMsg::KeyDown {
                        key: 0x61,
                        mask: 0,
                        button: 123,
                    }).unwrap();
                    self.tx.send(ActMsg::KeyUp {
                        key: 0x61,
                        mask: 0,
                        button: 123,
                    }).unwrap();
                }
            }
            ButtonState::Held => {
                if state == ButtonState::Down {
                    self.tx.send(ActMsg::KeyDown {
                        key: 0x62,
                        mask: 0,
                        button: 124,
                    }).unwrap();
                    self.tx.send(ActMsg::KeyUp {
                        key: 0x62,
                        mask: 0,
                        button: 124,
                    }).unwrap();
                }        
            }
        }
    }
}
