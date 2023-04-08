use std::sync::{mpsc::SyncSender, Mutex};

use anyhow::Result;
use barrier::ActMsg;
use const_env::from_env;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use keycodes::ASCII_2_HID;
use lazy_static::lazy_static;
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

pub const INIT_USB: bool = true;

// M5Atom S3 Lite has a status NeoPixel on GPIO 35
#[from_env("STATUS_LED_PIN")]
const STATUS_LED_PIN: u32 = 35;

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

    let b = PasteButton {
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

struct PasteButton {
    tx: SyncSender<ActMsg>,
}

impl PasteButton {
    fn send_char(&self, c: char) {
        if c > 0x7F as char {
            return;
        }
        let byte = c as u8;
        let [k, m] = ASCII_2_HID[byte as usize];
        if k == 0 {
            return;
        }
        if m != 0 {
            self.tx.send(ActMsg::HidKeyDown { key: m }).ok();
            self.tx.send(ActMsg::HidKeyDown { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: m }).ok();
        } else {
            self.tx.send(ActMsg::HidKeyDown { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: k }).ok();
        }
    }
}

impl ButtonCallback for PasteButton {
    fn on_button_event(&mut self, state: ButtonState) {
        match state {
            ButtonState::Up => {}
            ButtonState::Down => {
                let s = {
                    let data = CLIPBOARD.lock().unwrap();
                    String::from_utf8_lossy(&data).to_string()
                };
                for c in s.chars() {
                    self.send_char(c);
                }
            }
            ButtonState::Held => {}
        }
    }
}
