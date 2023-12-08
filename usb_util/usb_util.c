/*
 * SPDX-FileCopyrightText: 2022 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Unlicense OR CC0-1.0
 */

#include <stdlib.h>
#include <device/usbd_pvt.h>
#include "esp_log.h"
#include "tinyusb.h"
#include "class/hid/hid.h"
#include "class/hid/hid_device.h"
#include "sdkconfig.h"
#include "descriptors_control.h"

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

enum {
    RID_KEYBOARD = 1,
    RID_MOUSE,
    RID_CONSUMER_CONTROL,
};

/**
 * @brief HID report descriptor
 *
 * In this example we implement Keyboard + Mouse HID device,
 * so we must define both report descriptors
 */
const uint8_t hid_report_descriptor[] = {
    TUD_HID_REPORT_DESC_KEYBOARD(HID_REPORT_ID(RID_KEYBOARD)),
    TUD_HID_REPORT_DESC_MOUSE_ABS(HID_REPORT_ID(RID_MOUSE)),
    TUD_HID_REPORT_DESC_CONSUMER(HID_REPORT_ID(RID_CONSUMER_CONTROL))
    };

/**
 * @brief Configuration descriptor
 *
 * This is a simple configuration descriptor that defines 1 configuration and 1 HID interface
 */
static const uint8_t hid_configuration_descriptor[] = {
    // Configuration number, interface count, string index, total length, attribute, power in mA
    TUD_CONFIG_DESCRIPTOR(1, 1, 0, TUSB_DESC_TOTAL_LEN, TUSB_DESC_CONFIG_ATT_REMOTE_WAKEUP, 100),

    // Interface number, string index, boot protocol, report descriptor len, EP In address, size & polling interval
    TUD_HID_DESCRIPTOR(0, 0, true, sizeof(hid_report_descriptor), 0x81, 16, 1),
};

/********* TinyUSB HID callbacks ***************/

// // Invoked when received GET HID REPORT DESCRIPTOR request
// // Application return pointer to descriptor, whose contents must exist long enough for transfer to complete
uint8_t const *tud_hid_descriptor_report_cb(uint8_t instance) {
    // We use only one interface and one HID report descriptor, so we can ignore parameter 'instance'
    return hid_report_descriptor;
}

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

void usb_util_abs_mouse_report(uint8_t buttons, uint16_t x, uint16_t y, int8_t wheel, int8_t pan)
{
    hid_abs_mouse_report_t report =
        {
            .buttons = buttons,
            .x = x,
            .y = y,
            .wheel = wheel,
            .pan = pan};
    if (!initialized)
    {
        ESP_LOGI(TAG, "Buttons: %i, X: %i, Y: %i, Wheel: %i, Pan: %i", buttons, x, y, wheel, pan);
        return;
    }
    while(!tud_hid_n_ready(0));
    tud_hid_n_report(0, HID_PROTOCOL_MOUSE, &report, sizeof(report));
    while(!tud_hid_n_ready(0));
    tud_hid_n_report(0, 0, NULL, 0);
}

void usb_util_keyboard_report(uint8_t modifier, uint8_t *key_report)
{
    if (!initialized)
    {
        ESP_LOGI(TAG, "Modifier: %i, Button [%i, %i, %i, %i, %i, %i]", modifier, key_report[0], key_report[1], key_report[2], key_report[3], key_report[4], key_report[5]);
        return;
    }
    while(!tud_hid_n_ready(0));
    tud_hid_keyboard_report(HID_PROTOCOL_KEYBOARD, modifier, key_report);
    while(!tud_hid_n_ready(0));
    tud_hid_n_report(0, 0, NULL, 0);
}

void usb_util_consumer_report(uint16_t code)
{
    if (!initialized)
    {
        ESP_LOGI(TAG, "Consumer code: %i", code);
        return;
    }
    // 
    while(!tud_hid_n_ready(0));
    tud_hid_n_report(0, RID_CONSUMER_CONTROL, &code, 2);
    while(!tud_hid_n_ready(0));
    tud_hid_n_report(0, 0, NULL, 0);
}

void usb_util_init(void)
{
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