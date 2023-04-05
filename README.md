Esparrier
=========

Esparrier is a [Barrier](https://github.com/debauchee/barrier) client for ESP32S3.

## How to build

1. Install Rust toolchain.
2. Install Rust ESP32 tools:
    * `espup` - https://github.com/esp-rs/espup
    * `cargo-espflash` - https://github.com/esp-rs/espflash
    * Install Rust ESP toolchain with `espup install`
3. Set environment variable for Rust ESP toolchain:
    * `source $HOME/export-esp.sh`
4. Build and flash:
    1. Set following environment variables:
        * `export WIFI_SSID="YOUR_WIFI_SSID"`
        * `export WIFI_PASSWORD="YOUR_WIFI_PASSWORD"`
        * `export BARRIER_SERVER="BARRIER_SERVER_IP"`
        * `export BARRIER_PORT="BARRIER_SERVER_PORT or use the default 24800"`
        * `export SCREEN_NAME="SCREEN_NAME"`
        * `export SCREEN_WIDTH="SCREEN_WIDTH"`
        * `export SCREEN_HEIGHT="SCREEN_HEIGHT"`
        * `export REVERSED_WHEEL="1 to reverse the mouse wheel, 0 to use the default"`
        * `export V_SCROLL_SCALE="FLOAT_NUMBER_TO_SCALE_VERTICAL_MOUSE_WHEEL e.g. 1.0"`
        * `export H_SCROLL_SCALE="FLOAT_NUMBER_TO_SCALE_HORIZONTAL_MOUSE_WHEEL e.g. 1.0"`
    2. Put your board in the download mode, then build and flash with `cargo run --release`.

## Run

1. Configure Barrier server to accept the screen name you set in the environment variable `SCREEN_NAME`, and make sure you turn off the TLS.
2. Plug the board into the USB port.
3. The LED should be red on start, then turn blue when the board is connected to the WiFi, and finally turn dim yellow when the board is connected to the Barrier server.
4. When Barrier enters the screen, the LED turns bright green, and when Barrier leaves the screen, the LED turns dim yellow.
5. The board emulates a standard keyboard and an absolute mouse, it should work in any OS.
6. USB HID boot protocol is used, so you can should be able to use the board as a USB keyboard/mouse in BIOS/EFI or even if the OS doesn't have a driver for it.

## NOTES:

This program is only for testing purpose. It is not a complete implementation of Barrier client. There could be a lot of bugs and missing features.

* This code is developed and tested on [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite), other ESP32S3 boards may not work, or you need to change the code.
* The code should be working on ESP32S2, but some changes may be needed. It won't work on ESP8266/ESP32/ESP32C3 because they don't have required USB features.
* It doesn't support TLS, so you must run Barrier server without TLS.
* The mouse is configured to the absolute mode, you must set the correct screen resolution before building, otherwise the mouse may not work properly.
* Clipboard, file transfer, and cross-screen drag and drop are not supported due to the technical limitation, there is no way a standard USB HID device can do that, maybe an auxiliary app running on the host can help but I still don't have clear idea.
* Auto-switching doesn't work sometimes, you may need to configure hotkey on the Barrier server to switch screens manually.
* Media keys and Mac special keys are WIP, they need another Consumer Control device in addition to the HID keyboard/mouse.
* Frequently connect/disconnect may cause the board fail to connect to the WiFi and/or Barrier server, you may need to power off the board and wait for a while before trying again.
* In theory the board should be working with [InputLeap](https://github.com/input-leap/input-leap) server as well but I've never tested it.
* The USB VID/PID are randomly picked and not registered, so you may need to change the code to use your own VID/PID.
* The USB device is initialized *after* the board successfully connects to the WiFi and Barrier server, it may be too late to use the board as a USB keyboard/mouse in BIOS/EFI, some main board that has always-on USB ports may work, but I haven't tested it, or you can use a USB hub that can supply power even if the host is off.

## TODO:

- [ ] Support TLS
- [ ] Support media keys
- [ ] Support Mac special keys
- [ ] Support clipboard, maybe with a separate app running on the host to handle the clipboard data
