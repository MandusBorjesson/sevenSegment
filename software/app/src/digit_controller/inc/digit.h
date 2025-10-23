#ifndef DIGIT_H
#define DIGIT_H

#include <zephyr/drivers/gpio.h>

/*
 * Segment locations
 *
 *  ---A---
 * :       :
 * F       B
 * :       :
 *  ---G---
 * :       :
 * E       C
 * :       :
 * '---D---'
 *
 * */

struct digit_ctx_t {
    const struct gpio_dt_spec *control;
    uint8_t state;
};

int digit_init(struct digit_ctx_t *ctx);
int digit_set_segments(struct digit_ctx_t *ctx, uint8_t new_state);
int digit_set_number(struct digit_ctx_t *ctx, uint8_t number);
int digit_clear(struct digit_ctx_t *ctx);

#endif  // DIGIT_H
