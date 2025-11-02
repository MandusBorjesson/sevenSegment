#include "digit.h"

#include <zephyr/logging/log.h>

#define UPDATE_MASK_ALL 0x7F
#define STATE_INITIALIZED_BIT 0x80

LOG_MODULE_REGISTER(digit_controller, LOG_LEVEL_DBG);
#define N_SEGMENTS 7

/*
 * .------------------------------------------------------------.
 * : Pin      : 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16  :
 * :------------------------------------------------------------:
 * : Function : X  X  C  S  C  S  S  C  C  S  S  C  S  S  C  C  :
 * : Segment  : X  X  D  D  E  C  E  C  F  B  F  B  G  A  G  A  :
 * '------------------------------------------------------------'
 */

typedef struct segment_t
{
    size_t set;
    size_t clr;
} segment_t;

const segment_t _segments[N_SEGMENTS] = {
    { .set = 14, .clr = 16, }, // A
    { .set = 10, .clr = 12, }, // B
    { .set = 6,  .clr = 8,  }, // C
    { .set = 4,  .clr = 3,  }, // D
    { .set = 7,  .clr = 5,  }, // E
    { .set = 11, .clr = 9,  }, // F
    { .set = 13, .clr = 15, }, // G
};

size_t pin_to_offset(size_t pin) {
    // Offsets from shift registers
    if (pin >= 10) {
        return pin - 1;
    }
    return pin - 2;
};

void digit_init(struct digit_ctx_t *ctx) {
    ctx->state &= ~STATE_INITIALIZED_BIT;
}

bool digit_set_segments(struct digit_ctx_t *ctx, uint8_t new_state, uint16_t *value) {
    uint8_t update_mask = UPDATE_MASK_ALL;
    uint8_t old_state = ctx->state & ~STATE_INITIALIZED_BIT;

    new_state &= ~STATE_INITIALIZED_BIT;

    if ((ctx->state & STATE_INITIALIZED_BIT) > 0) {
        update_mask = old_state ^ new_state;
    }

    if (!update_mask) {
        LOG_DBG("No update required");
        return false;
    }

    LOG_DBG("Updating state from 0x%02x to 0x%02x, mask: 0x%02x",
            old_state,
            new_state,
            update_mask);
    *value = 0;

    for (int i = 0; i < N_SEGMENTS; i++) {
        // Should segment be updated?
        if(update_mask & (1<<i)) {
            if (new_state & (1<<i)) {
                *value |= 1 << pin_to_offset(_segments[i].set);
            } else {
                *value |= 1 << pin_to_offset(_segments[i].clr);
            }
        }
    }

    LOG_DBG("Update data: 0x%04x", *value);

    ctx->state = new_state | STATE_INITIALIZED_BIT;
    return true;
}

bool digit_set_number(struct digit_ctx_t *ctx, uint8_t number, uint16_t *value) {
    const uint8_t states[] = {
        // GFEDCBA
        0b00111111,
        0b00000110,
        0b01011011,
        0b01001111,
        0b01100110,
        0b01101101,
        0b01111101,
        0b00000111,
        0b01111111,
        0b01101111,
    };

    if (number >= sizeof(states)/sizeof(*states)) {
        LOG_ERR("Requested number %d larger than what can be displayed!", number);
        return false;
    }

    return digit_set_segments(ctx, states[number], value);
}

bool digit_clear(struct digit_ctx_t *ctx, uint16_t *value) {
    return digit_set_segments(ctx, 0x00, value);
}
