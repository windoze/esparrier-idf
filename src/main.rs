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
mod status;
mod usb_actor;
mod utils;

use paste_button::start_paste_button_task;
use settings::*;
use utils::*;

use crate::{
    barrier::ThreadedActuator,
    status::{set_status, Status},
    usb_actor::UsbHidActuator,
};

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

    let peripherals = Peripherals::take().unwrap();

    #[cfg(feature = "m5atoms3")]
    {
        let display = {
            // backlight
            esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio16)
                .unwrap()
                .set_high()
                .unwrap();
    
            let di = display_interface_spi::SPIInterfaceNoCS::new(
                esp_idf_hal::spi::SpiDeviceDriver::new_single(
                    peripherals.spi2,
                    peripherals.pins.gpio17,
                    peripherals.pins.gpio21,
                    Option::<esp_idf_hal::gpio::Gpio21>::None,
                    esp_idf_hal::spi::Dma::Disabled,
                    Some(peripherals.pins.gpio15),
                    &esp_idf_hal::spi::SpiConfig::new()
                        .baudrate(esp_idf_hal::units::FromValueType::MHz(27u32).into()),
                )
                .unwrap(),
                esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio33).unwrap(),
            );
    
            mipidsi::Builder::st7789(di)
                .with_color_order(mipidsi::ColorOrder::Bgr)
                .with_display_size(128, 128)
                .with_framebuffer_size(128, 128)
                .with_window_offset_handler(|_| (2, 1))
                .init(
                    &mut esp_idf_hal::delay::Ets,
                    Some(esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio34).unwrap()),
                )
                .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?
        };
        status::lcd_status::init_display(display);
    }

    set_status(Status::Start);

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
    for _ in 0..10 {
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
        info!("Reconnecting in 1 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    info!("Failed to connect to barrier, restarting...");
    set_status(Status::WifiConnected); // set_led(RGB { r: 0, g: 0, b: 255 });

    panic!("Disconnected, restarting...")
}
