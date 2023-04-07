use std::{sync::Mutex, time::Duration};

use anyhow::{bail, Result};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use log::info;

use embedded_svc::wifi::{AccessPointConfiguration, ClientConfiguration, Configuration, Wifi};

use smart_leds::{SmartLedsWrite, RGB};
use ws2812_esp32_rmt_driver::{driver::color::LedPixelColorImpl, LedPixelEsp32Rmt};

use crate::settings::{get_wifi_ssid, get_wifi_password};

pub static STATUS_LED: Mutex<
    Option<LedPixelEsp32Rmt<RGB<u8>, LedPixelColorImpl<3, 1, 0, 2, 255>>>,
> = Mutex::new(None);

pub fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    use std::net::Ipv4Addr;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    unsafe { esp_idf_sys::esp_wifi_set_ps(0) };

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ssid = get_wifi_ssid();
    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: ssid.into(),
            password: get_wifi_password().into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.start()?;

    info!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    info!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}

pub fn set_led(color: RGB<u8>) {
    if let Some(ws2812) = STATUS_LED.lock().unwrap().as_mut() {
        ws2812.write(std::iter::once(color)).unwrap()
    }
}
