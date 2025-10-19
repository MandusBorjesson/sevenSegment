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

LOG_MODULE_DECLARE(app, LOG_LEVEL_DBG);

/* 1000 msec = 1 sec */
#define SLEEP_TIME_MS   1000

/* The devicetree node identifier for the "led0" alias. */
#define LED0_NODE DT_ALIAS(led0)

/*
 * A build error on this line means your board is unsupported.
 * See the sample documentation for information on how to fix this.
 */
static const struct gpio_dt_spec led = GPIO_DT_SPEC_GET(DT_NODELABEL(relay_0), gpios);

// static const struct device *const spi_dev = DEVICE_DT_GET(DT_NODELABEL(spi2));
static const struct device *const spi_dev = DEVICE_DT_GET(DT_ALIAS(shift_register_spi));

static int setup_spi(const struct device *dev) {
    printf("%s: setting up...\n", dev->name);

	if (!device_is_ready(dev)) {
		printf("%s: device not ready.\n", dev->name);
		return -ENODEV;
	}
    return 0;
}

int main(void)
{
	int ret;
	bool led_state = true;

	printf("Starting system...\n");
	if (!gpio_is_ready_dt(&led)) {
		return 0;
	}

    ret = setup_spi(spi_dev);
    if (ret) {
        printf("SPI device setup failed\n");
        return ret;
    }

	ret = gpio_pin_configure_dt(&led, GPIO_OUTPUT_ACTIVE);
	if (ret < 0) {
		return 0;
	}
	printf("Initialized succesfully, entering loop\n");

	while (1) {
		ret = gpio_pin_toggle_dt(&led);
		if (ret < 0) {
			return 0;
		}

		led_state = !led_state;
		printf("LED state: %s\n", led_state ? "ON" : "OFF");

	    struct spi_config config = {
	        .frequency = 125000,
	        .operation = SPI_OP_MODE_MASTER | SPI_WORD_SET(9),
	        .slave = 0,
        };

	    uint8_t buff[10] = { 0x01, 0x01, 0x00, 0xff, 0x00, 0xa5, 0x00, 0x00, 0x01, 0x02};

	    struct spi_buf tx_buf = {
            .buf = buff,
            .len = sizeof(buff) / sizeof(*buff)
        };
	    struct spi_buf_set tx_bufs = { .buffers = &tx_buf, .count = 1 };

	    ret = spi_write(spi_dev, &config, &tx_bufs);
        if (ret) {
            printf("SPI tx failed: %d\n", ret);
        }

		k_msleep(SLEEP_TIME_MS);
	}
	return 0;
}
