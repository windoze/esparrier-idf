use std::sync::Mutex;

use anyhow::Result;
use const_env::from_env;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys::{self as _, nvs_flash_init};
use lazy_static::lazy_static;
use log::{error, info};

mod barrier;
mod keycodes;
mod reports;
mod settings;
mod status;
mod usb_actor;
mod utils;

use settings::*;
use utils::*;

use crate::{
    status::{set_status, Status},
    usb_actor::UsbHidActuator,
};

#[from_env("DEBUG_INIT_USB")]
pub const INIT_USB: bool = true;

#[cfg(feature = "watchdog")]
use std::time::Duration;
#[cfg(feature = "watchdog")]
use enumset::enum_set;
#[cfg(feature = "watchdog")]
use esp_idf_hal::{task::watchdog::TWDTConfig, cpu::Core};
#[cfg(feature = "watchdog")]
#[from_env("WATCHDOG_TIMEOUT")]
const WATCHDOG_TIMEOUT: u64 = 15;

#[cfg(feature = "paste")]
mod paste_button;
#[cfg(feature = "paste")]
use esp_idf_hal::gpio::AnyInputPin;
#[cfg(feature = "paste")]
use paste_button::start_paste_button_task;
#[cfg(feature = "paste")]
use crate::barrier::ThreadedActuator;
// M5Atom S3 and Lite has a button on GPIO 41
#[cfg(feature = "paste")]
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

    #[cfg(feature = "watchdog")]
    let mut driver = esp_idf_hal::task::watchdog::TWDTDriver::new(
        peripherals.twdt,
        &TWDTConfig {
            duration: Duration::from_secs(WATCHDOG_TIMEOUT),  // TODO:
            panic_on_trigger: true,
            subscribed_idle_tasks: enum_set!(Core::Core0)
        },
    )?;
    #[cfg(feature = "watchdog")]
    let mut watchdog = driver.watch_current_task()?;
    
    #[cfg(feature = "m5atoms3")]
    {
        use esp_idf_hal::spi::config::DriverConfig;
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
                    Some(peripherals.pins.gpio15),
                    &DriverConfig::default(),
                    &esp_idf_hal::spi::SpiConfig::new()
                        .baudrate(esp_idf_hal::units::FromValueType::MHz(27u32).into()),
                )
                .unwrap(),
                esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio33).unwrap(),
            );

            mipidsi::Builder::st7789(di)
                .with_color_order(mipidsi::ColorOrder::Bgr)
                .with_invert_colors(mipidsi::ColorInversion::Inverted)
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

    #[cfg(feature = "paste")]
    let mut actor = {
        let actor = UsbHidActuator::new(screen_width, screen_height);
        let actor = ThreadedActuator::new(screen_width, screen_height, actor);
    
        start_paste_button_task(
            unsafe { AnyInputPin::new(PASTE_BUTTON_PIN) },
            actor.get_sender(),
        );
        actor
    };

    // Do not use paste button
    #[cfg(not(feature = "paste"))]
    let mut actor = UsbHidActuator::new(screen_width, screen_height);

    info!("Connecting to barrier...");
    for _ in 0..10 {
        match barrier::start(
            get_barrier_server(),
            get_barrier_port(),
            get_screen_name(),
            &mut actor,
            #[cfg(feature = "watchdog")]
            &mut watchdog,
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
