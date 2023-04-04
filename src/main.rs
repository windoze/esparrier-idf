use std::{time::Duration, thread::sleep};

use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use log::info;
use smart_leds::RGB;
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

mod barrier;
mod usb_actor;
mod utils;
mod keycodes;

use utils::*;

use crate::usb_actor::UsbHidActuator;

// M5Atom S3 Lite has a status NeoPixel on GPIO 35
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

    // let mut hue = unsafe { esp_random() } as u8;
    // loop {
    //     let pixels = std::iter::repeat(hsv2rgb(Hsv {
    //         hue,
    //         sat: 255,
    //         val: 8,
    //     }))
    //     .take(25);
    //     if let Ok(mut ws2812) = STATUS_LED.lock() {
    //         ws2812.as_mut().unwrap().write(pixels).unwrap();
    //     } 

    //     sleep(Duration::from_millis(100));

    //     hue = hue.wrapping_add(10);
    // }

    let mut actor = UsbHidActuator::new(2560, 1440);

    // Reconnect if disconnected
    loop {
        info!("Connecting to barrier...");
        match barrier::start("192.168.2.59", 24800, "ESPARRIER", &mut actor) {
            Ok(_) => {
                info!("Connection closed");
            }
            Err(e) => {
                info!("Connection failed: {}", e);
            }
        }
        set_led(RGB { r: 0, g: 0, b: 255 });
        sleep(Duration::from_millis(500));
    }

    // Ok(())
}
