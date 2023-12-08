Esparrier
=========

Esparrier is a [Barrier](https://github.com/debauchee/barrier) client for ESP32S3.

## How to build

1. Install Rust toolchain.
2. Install Rust ESP32 tools:
    * `espup` - https://github.com/esp-rs/espup
    * `ldproxy` - https://github.com/esp-rs/embuild
    * `cargo-espflash` - https://github.com/esp-rs/espflash, require version >= 2.0.0rc3
    * `espmonitor` - https://github.com/esp-rs/espmonitor
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
        * `export REVERSED_WHEEL="true to reverse the mouse wheel, false to use the default"`
        * `export V_SCROLL_SCALE="FLOAT_NUMBER_TO_SCALE_VERTICAL_MOUSE_WHEEL e.g. 1.0"`
        * `export H_SCROLL_SCALE="FLOAT_NUMBER_TO_SCALE_HORIZONTAL_MOUSE_WHEEL e.g. 1.0"`
    2. Put your board in the download mode, then build and flash with `cargo run --release`. On M5Atom S3 Lite, you need to hold the reset button until the green LED turns on, then release the button. And you need to press the reset button again after flashing to exit the download mode.

## Run

1. Configure Barrier server to accept the screen name you set in the environment variable `SCREEN_NAME`, and make sure you turn off the TLS.
2. Plug the board into the USB port.
3. The LED should be red on start, then turn blue when the board is connected to the WiFi, and finally turn dim yellow when the board is connected to the Barrier server.
4. When Barrier enters the screen, the LED turns bright green, and when Barrier leaves the screen, the LED turns dim yellow.
5. The board emulates a standard keyboard and an absolute mouse, it should work in any OS.
6. USB HID boot protocol is used, so you should be able to use the board as a USB keyboard/mouse in BIOS/EFI or even if the OS doesn't have a driver for it.

## Clipboard

The program now has limited support of the clipboard when the feature `paste` is enabled.

First you need to activate other screen and copy something into the clipboard, then switch to the screen connected to the board.

When the screen is activated, the board receives the clipboard content sent by the Barrier server, **keeps the first 1024 characters of the plain text format and discard everything else**.

Then you can "paste" the text by pressing the button on the board, the board will convert the text into a sequence of keystrokes, and send them to the computer. All characters except the visible ASCII codes will be discarded as they cannot be directly mapped to USB HID key codes, or they may have special meaning that can mess up things.

The program cannot "copy" content to the clipboard.

Better clipboard support is still WIP.

NOTE: When you copied a large amount of text or big image from other screen then moved into the screen connected to the board, the board may stuck for a while, this is because the board is trying to discard the clipboard content. Even it will not parse and hold the whole content, still it needs to receive the whole content from the Barrier server as there is no way to skip a chunk in the middle of a TCP stream without actually reading it. But the board should resume operation after few seconds and it will not repeatedly process the same clipboard content if you move out and move in again.

## Update Configurations

First, you need to install some tools:

* ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/get-started/index.html.
* `esptool.py`: can be installed with `pip install esptool`.

### Prepare and Update Configurations

1. Create a CSV file, refer to [settings.csv](settings.csv) for the format. You need to retain **all** rows in the config file, only change the values in the right most column. Be aware that some keys have different values in the config file and the environment variables.
    * The value `reversed_wheel` of is used to reverse the mouse wheel, `1` to reverse, `0` to use the default.
    * The value `h_scroll_scale` and `v_scroll_scale` have scale of 100, `100` means `1` and `80` means `0.8`, etc.
2. Use `nvs_partition_gen.py` comes with ESP-IDF to generate a partition table with NVS partition.
    ```bash
    python /PATH/TO/ESP-IDF/components/nvs_flash/nvs_partition_generator/nvs_partition_gen.py generate "YOUR_CSV_FILE.csv" settings.bin 0x6000
    ```
3. Put the board into the download mode, then use `esptool.py` to flash the NVS partition.
    ```bash
    esptool.py --chip esp32s3 --port /dev/ttyUSB0 --baud 921600 write_flash 0x9000 settings.bin
    ```
4. Exit the download mode and reset the board, the new configurations should be applied.

## Build for other ESP32S3 boards

1. You need to disable the default feature, e.g. `cargo build --release --no-default-features`, this disables both LED and the paste button, but other functions remain.
2. If there is a button on the board, you can enable the feature `paste` to support clipboard, and you need to set the environment `PASTE_BUTTON_PIN` to the correct pin number, on M5AtomS3/Lite, it's 41.
3. If there is a RGB LED (WS2812B) on the board, you can use `m5atoms3lite` feature to enable the LED, and you need to set the environment `STATUS_LED_PIN` to the correct pin number, on M5AtomS3/Lite, it's 35, on M5StampS3, it's 21.

## NOTES:

**WARNING**: This program is only for testing purpose. It is not a complete implementation of Barrier client. There could be a lot of bugs and missing features. It has no concept of security, neither on the WiFi nor on the USB. It is not recommended to use it in anywhere but a private environment.

* This code is developed and tested on [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite), other ESP32S3 boards may not work, or you need to change the code.
* The code should be working on ESP32S2, but some changes may be needed. It won't work on ESP8266/ESP32/ESP32C3 because they don't have required USB features.
* It doesn't support TLS, so you must run Barrier server without TLS.
* The mouse is configured to the absolute mode, you must set the correct screen resolution before building, otherwise the mouse may not work properly.
* Clipboard, file transfer, and cross-screen drag and drop are not supported due to the technical limitation, there is no way a standard USB HID device can do that, maybe an auxiliary app running on the host can help but I still don't have clear idea.
* Auto-switching doesn't work properly unless you set the screen size correctly, otherwise you may need to configure hotkey on the Barrier server to switch screens manually.
* Frequently connect/disconnect may cause the board fail to connect to the WiFi and/or Barrier server, you may need to power off the board and wait for a while before trying again.
* In theory the board should be working with [InputLeap](https://github.com/input-leap/input-leap) server as well but I've never tested it.
* The USB VID/PID are randomly picked and not registered, so you may need to change the code to use your own VID/PID.
* The USB remote wakeup may not work because the standard forbids a suspended device consume too much current but this program needs much more than the standard says to keep Wi-Fi connected. I still haven't figured out how to keep the program running with the current <2.5mA. Of course you can choose a board with external power source such as a battery, but it seems to be an overkill.
* The program can accept inputs only **after** the board successfully connects to the WiFi and Barrier server, it may be too late to use the board as a USB keyboard/mouse in BIOS/EFI, some main board that has always-on USB ports may work, but I haven't tested it, or you can use a USB hub that can supply power even if the host is off.
* By default the `watchdog` feature is enabled, which can be optionally disabled. The watchdog will reset the board if it doesn't receive heartbeat from the Barrier server, or the program itself runs out of control and doesn't process the heartbeat, for the number of seconds defined in `WATCHDOG_TIMEOUT` environment variable. The default watchdog timeout is 15 seconds, as the default Barrier heartbeat interval is 5 seconds, you may need to change the watchdog timeout if the Barrier server has a long heartbeat interval.

## TODO:

- [x] Support media keys
- [x] Re-configure without rebuilding
- [x] Support other ESP32S3 boards
- [ ] Support Mac special keys
- [ ] Support TLS
- [ ] NVS encryption
- [ ] OTA update
- [ ] Support clipboard, maybe with a separate app running on the host to handle the clipboard data
