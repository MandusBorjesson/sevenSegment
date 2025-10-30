#ifndef DIGIT_H
#define DIGIT_H

#include <stdint.h>
#include <stdbool.h>

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
    uint8_t state;
};

void digit_init(struct digit_ctx_t *ctx);
bool digit_set_segments(struct digit_ctx_t *ctx, uint8_t new_state, uint16_t *value);
bool digit_set_number(struct digit_ctx_t *ctx, uint8_t number, uint16_t *value);
bool digit_clear(struct digit_ctx_t *ctx, uint16_t *value);

#endif  // DIGIT_H
