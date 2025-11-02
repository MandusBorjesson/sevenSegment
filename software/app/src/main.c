/*
 * Copyright (c) 2016 Intel Corporation
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include <stdio.h>
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/logging/log.h>
#include "digit.h"

LOG_MODULE_REGISTER(app, LOG_LEVEL_INF);

#define N_DIGITS 4

typedef struct digit_control {
    struct digit_ctx_t ctx;
    const struct gpio_dt_spec *relay;
} digit_control;

static const struct gpio_dt_spec sr_oe = GPIO_DT_SPEC_GET(DT_NODELABEL(shift_register_oe), gpios);
static const struct gpio_dt_spec sr_clk = GPIO_DT_SPEC_GET(DT_NODELABEL(shift_register_clk), gpios);
static const struct gpio_dt_spec sr_mosi = GPIO_DT_SPEC_GET(DT_NODELABEL(shift_register_mosi), gpios);

static const struct gpio_dt_spec _relay_0 = GPIO_DT_SPEC_GET(DT_NODELABEL(relay_0), gpios);
static const struct gpio_dt_spec _relay_1 = GPIO_DT_SPEC_GET(DT_NODELABEL(relay_1), gpios);
static const struct gpio_dt_spec _relay_2 = GPIO_DT_SPEC_GET(DT_NODELABEL(relay_2), gpios);
static const struct gpio_dt_spec _relay_3 = GPIO_DT_SPEC_GET(DT_NODELABEL(relay_3), gpios);

static digit_control digits[N_DIGITS] = {
    {
        .relay = &_relay_0,
    },
    {
        .relay = &_relay_1
    },
    {
        .relay = &_relay_2
    },
    {
        .relay = &_relay_3
    },
};

static int setup_spi(const struct gpio_dt_spec *spi_oe, const struct gpio_dt_spec *spi_mosi, const struct gpio_dt_spec *spi_clk) {

    LOG_INF("Setting up SPI...");

    if (!gpio_is_ready_dt(spi_oe)) {
        return -ENODEV;
    }
    int ret = gpio_pin_configure_dt(spi_oe, GPIO_OUTPUT_HIGH);
    if (ret < 0) {
        return ret;
    }

    if (!gpio_is_ready_dt(spi_mosi)) {
        return -ENODEV;
    }
    ret = gpio_pin_configure_dt(spi_mosi, GPIO_OUTPUT_LOW);
    if (ret < 0) {
        return ret;
    }

    if (!gpio_is_ready_dt(spi_clk)) {
        return -ENODEV;
    }
    ret = gpio_pin_configure_dt(spi_clk, GPIO_OUTPUT_LOW);
    if (ret < 0) {
        return ret;
    }

    return 0;
}

static int setup_digit(digit_control* digit) {
    if (!gpio_is_ready_dt(digit->relay)) {
        return -ENODEV;
    }
    int ret = gpio_pin_configure_dt(digit->relay, GPIO_OUTPUT_LOW);
    if (ret < 0) {
        return ret;
    }

    digit_init(&digit->ctx);
    return 0;
};

static int update_shift_registers(const struct gpio_dt_spec *spi_mosi, const struct gpio_dt_spec *spi_clk, uint16_t data) {
    for (int i = 0; i < 16; i++) {
        int bit = (1<<(15-i)) & data;
        gpio_pin_set_dt(spi_mosi, bit > 0);
        k_sleep(K_USEC(1));
        gpio_pin_set_dt(spi_clk, 1);
        k_sleep(K_USEC(1));
        gpio_pin_set_dt(spi_clk, 0);
        k_sleep(K_USEC(1));
    }
    gpio_pin_set_dt(spi_mosi, 0);
    k_sleep(K_USEC(1));
    gpio_pin_set_dt(spi_clk, 1);
    k_sleep(K_USEC(1));
    gpio_pin_set_dt(spi_clk, 0);
    k_sleep(K_USEC(1));
    return 0;
}

#define RETURN_ON_ERR(_ret, func, ...)              \
    _ret = func(__VA_ARGS__);                       \
    if (_ret < 0) {                                 \
        LOG_ERR(#func ": failed with: %d", ret);    \
        return _ret;                                \
    }

#define BREAK_ON_ERR(_ret, func, ...)               \
    _ret = func(__VA_ARGS__);                       \
    if (_ret < 0) {                                 \
        LOG_ERR(#func ": failed with: %d", ret);    \
        break;                                      \
    }


int main(void)
{
    int ret;
    uint16_t count = 990;
    uint16_t current_value = 0;
    bool needs_update = false;
    uint16_t register_values = 0;

    LOG_INF("Starting system...");

    RETURN_ON_ERR(ret, setup_spi, &sr_oe, &sr_mosi, &sr_clk);

    for (int i = 0; i < N_DIGITS; i++) {
        ret = setup_digit(&digits[i]);
        if (ret) {
            LOG_INF("Digit %d setup failed: %d", i, ret);
            return ret;
        }
    }

    LOG_INF("Initialized succesfully, entering loop");

    while (1) {
        current_value = count;

        LOG_INF("Displaying value: %d", current_value);
        for (int i = 0; i < N_DIGITS; i++) {
            digit_control* digit = &digits[i];

            // Update shift registers
            if (i > 0 && current_value == 0) {
                LOG_INF("  D%d: Clearing digit", i);
                needs_update = digit_set_segments(&digit->ctx, 0x00, &register_values);
            } else {
                LOG_INF("  D%d: Setting digit to %d...", i, current_value % 10);
                needs_update = digit_set_number(&digit->ctx, current_value % 10, &register_values);
            }

            current_value /= 10;

            if (!needs_update) {
                LOG_DBG("    D%d: No update", i);
                continue;
            }

            LOG_DBG("    D%d: Committing data: 0x%04x", i, register_values);

            BREAK_ON_ERR(ret, update_shift_registers, &sr_mosi, &sr_clk, register_values);

            // Enable segment relay
            BREAK_ON_ERR(ret, gpio_pin_set_dt, digit->relay, 1);
            k_sleep(K_MSEC(100));

            // Commit segment changes
            BREAK_ON_ERR(ret, gpio_pin_set_dt, &sr_oe, 0);
            k_sleep(K_MSEC(200));

            BREAK_ON_ERR(ret, gpio_pin_set_dt, &sr_oe, 1);
            BREAK_ON_ERR(ret, gpio_pin_set_dt, digit->relay, 0);
        }
        if (++count > 9999) {
            count = 0;
        }
        k_sleep(K_SECONDS(1));
    }
    return 0;
}
