use anyhow::Result;
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::{BlockingWifi, EspWifi},
};
use esp_idf_sys::esp_pm_configure;
use log::info;

use embedded_svc::wifi::{ClientConfiguration, Configuration};

use crate::settings::{get_wifi_password, get_wifi_ssid};

pub fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    unsafe {
        let cfg = esp_idf_sys::esp_pm_config_esp32s3_t {
            max_freq_mhz: 240,
            min_freq_mhz: 160,
            light_sleep_enable: false,
        };
        let ret = esp_pm_configure(&cfg as *const _ as *const std::ffi::c_void);
        info!("Set power management configuration: {}", ret);
    }
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");

    wifi.start()?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: get_wifi_ssid().into(),
        password: get_wifi_password().into(),
        channel: None,
        ..Default::default()
    }))?;

    info!("Connecting wifi...");

    wifi.connect()?;

    info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    unsafe {
        let ret = esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
        info!("Set WiFi power management configuration: {}", ret);
    }

    Ok(Box::new(esp_wifi))
}
