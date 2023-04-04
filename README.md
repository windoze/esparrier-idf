Esparrier
=========

Esparrier is a Barrier client for ESP32S3.

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

## NOTES:

This program is only for testing purpose. It is not a complete implementation of Barrier client. There could be a lot of bugs and missing features.

* This code is only tested on M5Atom S3 Lite, other ESP32S3 boards may not work, or you need to change the code.
* The code should be working on ESP32S2, but some changes may be needed. It won't work on ESP8266/ESP32/ESP32C3 because they don't have required USB features.
* It doesn't support TLS, so you must run Barrier server without TLS.
* The mouse is configured to the absolute mode, it means that you must set the correct screen resolution before building, otherwise the mouse may not work properly.
* Clipboard, file transfer, and cross-screen drag and drop are not supported.
* There are some significant performance issues, you may experience laggy mouse and keyboard input, or missing key and mouse events.
* Auto-switching doesn't work sometimes, you may need to configure hotkey on the Barrier server to switch screens manually.
