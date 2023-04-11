use std::ffi::{CStr, CString};

use const_env::from_env;
use esp_idf_sys::{
    nvs_flash_init, nvs_get_str, nvs_get_u16, nvs_handle_t, nvs_open,
    nvs_open_mode_t_NVS_READONLY, ESP_OK,
};
use lazy_static::lazy_static;

// Default values are from env
#[from_env("SCREEN_WIDTH")]
const DEFAULT_SCREEN_WIDTH: u16 = 1920;
#[from_env("SCREEN_HEIGHT")]
const DEFAULT_SCREEN_HEIGHT: u16 = 1080;
#[from_env("BARRIER_SERVER")]
const DEFAULT_BARRIER_SERVER: &str = "127.0.0.1";
#[from_env("BARRIER_PORT")]
const DEFAULT_BARRIER_PORT: u16 = 24800;
#[from_env("SCREEN_NAME")]
const DEFAULT_SCREEN_NAME: &str = "ESPARRIER";
const DEFAULT_SSID: &str = env!("WIFI_SSID");
const DEFAULT_PASS: &str = env!("WIFI_PASSWORD");
#[from_env("REVERSED_WHEEL")]
const DEFAULT_REVERSED_WHEEL: bool = false;
#[from_env("V_SCROLL_SCALE")]
const DEFAULT_V_SCROLL_SCALE: f32 = 1.0;
#[from_env("H_SCROLL_SCALE")]
const DEFAULT_H_SCROLL_SCALE: f32 = 1.0;

lazy_static! {
    static ref NVS_HANDLE: nvs_handle_t = {
        unsafe { nvs_flash_init() };
        let mut handle: nvs_handle_t = 0;
        let name = CString::new("settings").unwrap();
        unsafe {
            if nvs_open(
                name.as_ptr(),
                nvs_open_mode_t_NVS_READONLY,
                &mut handle,
            ) != ESP_OK
            {
                handle = 0;
            }
        };
        handle
    };
    static ref SCREEN_WIDTH: u16 = get_u16("screen_width").unwrap_or(DEFAULT_SCREEN_WIDTH);
    static ref SCREEN_HEIGHT: u16 = get_u16("screen_height").unwrap_or(DEFAULT_SCREEN_HEIGHT);
    static ref BARRIER_SERVER: &'static str = get_str("barrier_server").unwrap_or(DEFAULT_BARRIER_SERVER);
    static ref BARRIER_PORT: u16 = get_u16("barrier_port").unwrap_or(DEFAULT_BARRIER_PORT);
    static ref SCREEN_NAME: &'static str = get_str("screen_name").unwrap_or(DEFAULT_SCREEN_NAME);
    static ref SSID: &'static str = get_str("ssid").unwrap_or(DEFAULT_SSID);
    static ref PASS: &'static str = get_str("pass").unwrap_or(DEFAULT_PASS);
    static ref REVERSED_WHEEL: bool = get_bool("reversed_wheel").unwrap_or(DEFAULT_REVERSED_WHEEL);
    static ref V_SCROLL_SCALE: f32 = get_f32("v_scroll_scale").unwrap_or(DEFAULT_V_SCROLL_SCALE);
    static ref H_SCROLL_SCALE: f32 = get_f32("h_scroll_scale").unwrap_or(DEFAULT_H_SCROLL_SCALE);
}

fn get_bool(key: &str) -> Option<bool> {
    get_u16(key).map(|v| v != 0)
}

fn get_u16(key: &str) -> Option<u16> {
    if *NVS_HANDLE == 0 {
        return None;
    }
    let key = CString::new(key).unwrap();
    let mut out_value: u16 = 0;
    let ret = unsafe { nvs_get_u16(*NVS_HANDLE, key.as_ptr(), &mut out_value) };
    if ret == ESP_OK {
        Some(out_value)
    } else {
        None
    }
}

fn get_str(key: &str) -> Option<&'static str> {
    if *NVS_HANDLE == 0 {
        return None;
    }
    let key = CString::new(key).unwrap();
    let mut out_value: [i8; 64] = [0; 64];
    let mut size = 64;
    let ret = unsafe { nvs_get_str(*NVS_HANDLE, key.as_ptr(), out_value.as_mut_ptr(), &mut size) };
    if ret == ESP_OK {
        // WARN: Deliberately leak the string, so you shouldn't repeatedly call this function for the same key
        Some(Box::leak(
            unsafe { CStr::from_ptr(out_value.as_ptr()) }
                .to_string_lossy()
                .to_string()
                .into_boxed_str(),
        ))
    } else {
        None
    }
}

fn get_f32(key: &str) -> Option<f32> {
    if *NVS_HANDLE == 0 {
        return None;
    }
    let key = CString::new(key).unwrap();
    let mut out_value: u16 = 0;
    let ret = unsafe { nvs_get_u16(*NVS_HANDLE, key.as_ptr(), &mut out_value) };
    if ret == ESP_OK {
        Some(out_value as f32 / 100.0)
    } else {
        None
    }
}

pub fn get_wifi_ssid() -> &'static str {
    *SSID
}

pub fn get_wifi_password() -> &'static str {
    *PASS
}

pub fn get_barrier_server() -> &'static str {
    *BARRIER_SERVER
}

pub fn get_barrier_port() -> u16 {
    *BARRIER_PORT
}

pub fn get_screen_name() -> &'static str {
    *SCREEN_NAME
}

pub fn get_screen_width() -> u16 {
    *SCREEN_WIDTH
}

pub fn get_screen_height() -> u16 {
    *SCREEN_HEIGHT
}

pub fn get_reversed_wheel() -> bool {
    *REVERSED_WHEEL
}

pub fn get_v_scroll_scale() -> f32 {
    *V_SCROLL_SCALE
}

pub fn get_h_scroll_scale() -> f32 {
    *H_SCROLL_SCALE
}
