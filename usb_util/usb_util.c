/*
 * SPDX-FileCopyrightText: 2022 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Unlicense OR CC0-1.0
 */

#include <stdlib.h>
#include <device/usbd_pvt.h>
// #include <hal/uart_types.h>
// #include <driver/uart.h>
#include "esp_log.h"
// #include "freertos/FreeRTOS.h"
// #include "freertos/task.h"
#include "tinyusb.h"
#include "class/hid/hid.h"
#include "class/hid/hid_device.h"
// #include "driver/gpio.h"
// #include "key_types.h"
// #include "led_strip.h"
#include "sdkconfig.h"

// // zero = no movement
// #define POINTER_POS_MIN_VAL 1
// #define POINTER_POS_MAX_VAL 32767 //  0x7fff according to usb spec
// #define ECHO_UART_PORT_NUM      (UART_NUM_0)
// #define ECHO_UART_BAUD_RATE     (460800)
// #define ECHO_TASK_STACK_SIZE    (2048)

// Mouse Report Descriptor Template
#define TUD_HID_REPORT_DESC_MOUSE_ABS(...)                                                        \
    HID_USAGE_PAGE(HID_USAGE_PAGE_DESKTOP),                                                       \
        HID_USAGE(HID_USAGE_DESKTOP_MOUSE),                                                       \
        HID_COLLECTION(HID_COLLECTION_APPLICATION), /* Report ID if any */                        \
        __VA_ARGS__                                                                               \
        HID_USAGE(HID_USAGE_DESKTOP_POINTER),                                                     \
        HID_COLLECTION(HID_COLLECTION_PHYSICAL),                                                  \
        HID_USAGE_PAGE(HID_USAGE_PAGE_BUTTON),                                                    \
        HID_USAGE_MIN(1),                                                                         \
        HID_USAGE_MAX(5),                                                                         \
        HID_LOGICAL_MIN(0),                                                                       \
        HID_LOGICAL_MAX(1), /* Left, Right, Middle, Backward, Forward buttons */                  \
        HID_REPORT_COUNT(5),                                                                      \
        HID_REPORT_SIZE(1),                                                                       \
        HID_INPUT(HID_DATA | HID_VARIABLE | HID_ABSOLUTE), /* 3 bit padding */                    \
        HID_REPORT_COUNT(1),                                                                      \
        HID_REPORT_SIZE(3),                                                                       \
        HID_INPUT(HID_CONSTANT),                                                                  \
        HID_USAGE_PAGE(HID_USAGE_PAGE_DESKTOP), /* X, Y position [0, 32767] */                    \
        HID_USAGE(HID_USAGE_DESKTOP_X),                                                           \
        HID_USAGE(HID_USAGE_DESKTOP_Y),                                                           \
        HID_LOGICAL_MIN(0),                                                                       \
        HID_LOGICAL_MAX_N(32767, 2),                                                              \
        HID_REPORT_COUNT(2),                                                                      \
        HID_REPORT_SIZE(16),                                                                      \
        HID_INPUT(HID_DATA | HID_VARIABLE | HID_ABSOLUTE), /* Verital wheel scroll [-127, 127] */ \
        HID_USAGE(HID_USAGE_DESKTOP_WHEEL),                                                       \
        HID_LOGICAL_MIN(0x81),                                                                    \
        HID_LOGICAL_MAX(0x7f),                                                                    \
        HID_REPORT_COUNT(1),                                                                      \
        HID_REPORT_SIZE(8),                                                                       \
        HID_INPUT(HID_DATA | HID_VARIABLE | HID_RELATIVE),                                        \
        HID_USAGE_PAGE(HID_USAGE_PAGE_CONSUMER), /* Horizontal wheel scroll [-127, 127] */        \
        HID_USAGE_N(HID_USAGE_CONSUMER_AC_PAN, 2),                                                \
        HID_LOGICAL_MIN(0x81),                                                                    \
        HID_LOGICAL_MAX(0x7f),                                                                    \
        HID_REPORT_COUNT(1),                                                                      \
        HID_REPORT_SIZE(8),                                                                       \
        HID_INPUT(HID_DATA | HID_VARIABLE | HID_RELATIVE),                                        \
        HID_COLLECTION_END,                                                                       \
        HID_COLLECTION_END

// #define APP_BUTTON (GPIO_NUM_0) // Use BOOT signal by default
static const char *TAG = "USB";

/************* TinyUSB descriptors ****************/

#define TUSB_DESC_TOTAL_LEN (TUD_CONFIG_DESC_LEN + CFG_TUD_HID * TUD_HID_DESC_LEN)

/**
 * @brief HID report descriptor
 *
 * In this example we implement Keyboard + Mouse HID device,
 * so we must define both report descriptors
 */
const uint8_t hid_report_descriptor[] = {
    TUD_HID_REPORT_DESC_KEYBOARD(HID_REPORT_ID(HID_PROTOCOL_KEYBOARD)),
    TUD_HID_REPORT_DESC_MOUSE_ABS(HID_REPORT_ID(HID_PROTOCOL_MOUSE))};

/**
 * @brief Configuration descriptor
 *
 * This is a simple configuration descriptor that defines 1 configuration and 1 HID interface
 */
static const uint8_t hid_configuration_descriptor[] = {
    // Configuration number, interface count, string index, total length, attribute, power in mA
    TUD_CONFIG_DESCRIPTOR(1, 1, 0, TUSB_DESC_TOTAL_LEN, TUSB_DESC_CONFIG_ATT_REMOTE_WAKEUP, 100),

    // Interface number, string index, boot protocol, report descriptor len, EP In address, size & polling interval
    TUD_HID_DESCRIPTOR(0, 0, false, sizeof(hid_report_descriptor), 0x81, 16, 10),
};

/********* TinyUSB HID callbacks ***************/

// // Invoked when received GET HID REPORT DESCRIPTOR request
// // Application return pointer to descriptor, whose contents must exist long enough for transfer to complete
// uint8_t const *tud_hid_descriptor_report_cb(uint8_t instance) {
//     // We use only one interface and one HID report descriptor, so we can ignore parameter 'instance'
//     return hid_report_descriptor;
// }

// Invoked when received GET_REPORT control request
// Application must fill buffer report's content and return its length.
// Return zero will cause the stack to STALL request
uint16_t tud_hid_get_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t report_type, uint8_t *buffer,
                               uint16_t reqlen)
{
    (void)instance;
    (void)report_id;
    (void)report_type;
    (void)buffer;
    (void)reqlen;

    return 0;
}

// Invoked when received SET_REPORT control request or
// received data on OUT endpoint ( Report ID = 0, Type = 0 )
void tud_hid_set_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t report_type, uint8_t const *buffer,
                           uint16_t bufsize)
{
}

typedef struct TU_ATTR_PACKED
{
    uint8_t buttons; /**< buttons mask for currently pressed buttons in the mouse. */
    uint16_t x;      /**< Current x of the mouse. */
    uint16_t y;      /**< Current y on the mouse. */
    int8_t wheel;    /**< Current delta wheel movement on the mouse. */
    int8_t pan;      // using AC Pan
} hid_abs_mouse_report_t;

static int initialized = 0;

static uint8_t _buttons = 0;
static uint16_t _x = 0;
static uint16_t _y = 0;

void usb_util_move_to_pos(uint16_t x, uint16_t y)
{
    _x = x;
    _y = y;
    hid_abs_mouse_report_t report =
        {
            .buttons = _buttons,
            .x = x,
            .y = y,
            .wheel = 0,
            .pan = 0};
    if (!initialized)
    {
        return;
    }
    tud_hid_n_report(0, HID_PROTOCOL_MOUSE, &report, sizeof(report));
}

static void set_mouse_buttons(uint8_t buttons)
{
    _buttons = buttons;
}

static void unset_mouse_buttons(uint8_t buttons)
{
    _buttons = _buttons & ~buttons;
}

void usb_util_mouse_button(uint8_t buttons)
{
    set_mouse_buttons(buttons);
    usb_util_move_to_pos(_x, _y);
}

void usb_util_mouse_button_up(uint8_t buttons)
{
    unset_mouse_buttons(buttons);
    usb_util_move_to_pos(_x, _y);
}

void usb_util_mouse_wheel(int8_t scroll, int8_t pan)
{
    hid_abs_mouse_report_t report =
        {
            .buttons = _buttons,
            .x = _x,
            .y = _y,
            .wheel = scroll,
            .pan = pan};
    if (!initialized)
    {
        return;
    }
    tud_hid_n_report(0, HID_PROTOCOL_MOUSE, &report, sizeof(report));
}

// #define button_state_count 6
static uint8_t server_button_state[0x200] = {0};
static uint8_t key_report[6] = {0};

// static void debug_buttons() {
//     for (int i = 0; i < button_state_count; i++) {
//         if (button_state[i] != 0) {
//             ESP_LOGI(TAG, "Button state %i = %i", i, button_state[i]);
//         }
//     }
// }

static uint8_t get_modifier() {
    uint8_t modifier = 0;
    for (int i = 0; i < 6; i++) {
        if (key_report[i] == 0) {
            continue;
        }
        if (key_report[i] == 0xE0) {
            modifier |= KEYBOARD_MODIFIER_LEFTCTRL;
        }
        if (key_report[i] == 0xE1) {
            modifier |= KEYBOARD_MODIFIER_LEFTSHIFT;
        }
        if (key_report[i] == 0xE2) {
            modifier |= KEYBOARD_MODIFIER_LEFTALT;
        }
        if (key_report[i] == 0xE3) {
            modifier |= KEYBOARD_MODIFIER_LEFTGUI;
        }
        if (key_report[i] == 0xE4) {
            modifier |= KEYBOARD_MODIFIER_RIGHTCTRL;
        }
        if (key_report[i] == 0xE5) {
            modifier |= KEYBOARD_MODIFIER_RIGHTSHIFT;
        }
        if (key_report[i] == 0xE6) {
            modifier |= KEYBOARD_MODIFIER_RIGHTALT;
        }
        if (key_report[i] == 0xE7) {
            modifier |= KEYBOARD_MODIFIER_RIGHTGUI;
        }
    }
    return modifier;
}

void usb_util_key_down(uint8_t key, uint16_t button)
{
    ESP_LOGD(TAG, ">>>> Key down");
    ESP_LOGD(TAG, "Key down: button: %i key: %i", button, key);
    if (key == 0)
    {
        return;
    }
    if (server_button_state[button] == key)
    {
        // Key already pressed
        return;
    }
    server_button_state[button] = key;

    ESP_LOGD(TAG, "Got keydown for %i", key);
    for (int i = 0; i < 6; i++)
    {
        if (key_report[i] == 0)
        {
            key_report[i] = key;
            break;
        }
    }

    for (int i = 0; i < 6; i++)
    {
        ESP_LOGD(TAG, "Button %i = %i", i, key_report[i]);
    }
    ESP_LOGD(TAG, "<<<< Key down");
    if (!initialized)
    {
        return;
    }
    tud_hid_keyboard_report(HID_PROTOCOL_KEYBOARD, get_modifier(), key_report);
}

void usb_util_key_up(uint16_t button)
{
    ESP_LOGD(TAG, ">>>> Key up");
    uint8_t key = server_button_state[button];
    ESP_LOGD(TAG, "Key up: button: %i", button);
    server_button_state[button] = 0;
    for (int i = 0; i < 6; i++)
    {
        if (key_report[i] == key)
        {
            key_report[i] = 0;
            goto done;
        }
    }
    ESP_LOGE(TAG, "Got keyup for key with no corresponding keydown? %i", key);
done:
    for (int i = 0; i < 6; i++)
    {
        ESP_LOGD(TAG, "Button %i = %i", i, key_report[i]);
    }
    ESP_LOGD(TAG, "<<<< Key up");

    if (!initialized)
    {
        return;
    }

    // static uint8_t empty_key_report[6] = {0};
    // tud_hid_keyboard_report(HID_PROTOCOL_KEYBOARD, 0, empty_key_report);

    tud_hid_keyboard_report(HID_PROTOCOL_KEYBOARD, get_modifier(), key_report);
}

void usb_util_init(void)
{
    // init_synergy_hid_key_table();
    // configure_led();
    // led_strip_set_pixel(led_strip, 0, 16, 16, 16);
    // led_strip_refresh(led_strip);
    // vTaskDelay(pdMS_TO_TICKS(50));

    ESP_LOGI(TAG, "USB initialization");

    const tinyusb_config_t tusb_cfg = {
        .descriptor = NULL,
        .string_descriptor = NULL,
        .external_phy = false,
        .config_descriptor = hid_configuration_descriptor,
    };

    ESP_ERROR_CHECK(tinyusb_driver_install(&tusb_cfg));
    ESP_LOGI(TAG, "USB initialization DONE");
    initialized = 1;
}