#include "digit.h"

#include <zephyr/logging/log.h>

#define UPDATE_MASK_ALL 0x7F
#define STATE_INITIALIZED_BIT 0x80

LOG_MODULE_REGISTER(digit_controller, LOG_LEVEL_INF);
#define N_SEGMENTS 7

const uint8_t set_indices[N_SEGMENTS] = {
    11, // A
    7,  // B
    3,  // C
    1,  // D
    4,  // E
    8,  // F
    10, // G
};

const uint8_t clr_indices[N_SEGMENTS] = {
    13, // A
    9,  // B
    5,  // C
    0,  // D
    2,  // E
    6,  // F
    12, // G
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
                *value |= 1 << set_indices[i];
            } else {
                *value |= 1 << clr_indices[i];
            }
        }
    }

    LOG_DBG("Update data: 0x%04x", *value);

    ctx->state = new_state | STATE_INITIALIZED_BIT;
    return true;
}

bool digit_set_number(struct digit_ctx_t *ctx, uint8_t number, uint16_t *value) {
    const uint8_t states[] = {
        // ABCDEFG
        0b01111110,
        0b00110000,
        0b01101101,
        0b01111001,
        0b00110011,
        0b01011011,
        0b01011111,
        0b01110000,
        0b01111111,
        0b01110011,
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
