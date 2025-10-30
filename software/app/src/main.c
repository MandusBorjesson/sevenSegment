/*
 * Copyright (c) 2016 Intel Corporation
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include <stdio.h>
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/drivers/spi.h>
#include <zephyr/logging/log.h>
#include "digit.h"

LOG_MODULE_REGISTER(app, LOG_LEVEL_INF);

/* The devicetree node identifier for the "led0" alias. */
#define LED0_NODE DT_ALIAS(led0)

#define N_DIGITS 4

typedef struct digit_control {
    struct digit_ctx_t ctx;
    const struct gpio_dt_spec *relay;
} digit_control;

static const struct device *const spi_dev = DEVICE_DT_GET(DT_ALIAS(shift_register_spi));
static const struct gpio_dt_spec sr_oe = GPIO_DT_SPEC_GET(DT_NODELABEL(shift_register_oe), gpios);
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

static int setup_spi(const struct device *spi_dev, const struct gpio_dt_spec *spi_oe) {

    LOG_INF("%s: setting up...", spi_dev->name);

    if (!device_is_ready(spi_dev)) {
        LOG_ERR("%s: device not ready.", spi_dev->name);
        return -ENODEV;
    }

    if (!gpio_is_ready_dt(spi_oe)) {
        return -ENODEV;
    }
    int ret = gpio_pin_configure_dt(spi_oe, GPIO_OUTPUT_ACTIVE);
    if (ret < 0) {
        return ret;
    }

    return 0;
}

static int setup_digit(digit_control* digit) {
    if (!gpio_is_ready_dt(digit->relay)) {
        return -ENODEV;
    }
    int ret = gpio_pin_configure_dt(digit->relay, GPIO_OUTPUT_ACTIVE);
    if (ret < 0) {
        return ret;
    }

    digit_init(&digit->ctx);
    return 0;
};

static int update_shift_registers(const struct device* spi_device, uint16_t data) {
    const struct spi_config config = {
        .frequency = 125000,
        .operation = SPI_OP_MODE_MASTER | SPI_WORD_SET(9),
        .slave = 0,
    };

    struct spi_buf spi_data = {
        .buf = &data,
        .len = sizeof(data)
    };

    struct spi_buf_set spi_buf = {
        .buffers = &spi_data,
        .count = 1
    };

    if (!spi_device) {
        return -ENODEV;
    }

    int ret = spi_write(spi_device, &config, &spi_buf);
    if (ret) {
        return ret;
    }
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
    uint16_t count = 0;
    uint16_t current_value = 0;
    bool needs_update = false;
    uint16_t register_values = 0;

    LOG_INF("Starting system...");

    RETURN_ON_ERR(ret, setup_spi, spi_dev, &sr_oe);

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

            BREAK_ON_ERR(ret, update_shift_registers, spi_dev, register_values);

            // Enable segment relay
            BREAK_ON_ERR(ret, gpio_pin_set_dt, digit->relay, 1);
            k_sleep(K_MSEC(100));

            // Commit segment changes
            BREAK_ON_ERR(ret, gpio_pin_set_dt, &sr_oe, 1);
            k_sleep(K_MSEC(100));

            BREAK_ON_ERR(ret, gpio_pin_set_dt, &sr_oe, 0);
            BREAK_ON_ERR(ret, gpio_pin_set_dt, digit->relay, 0);
        }
        if (++count > 9999) {
            count = 0;
        }
        k_sleep(K_SECONDS(1));
    }
    return 0;
}
