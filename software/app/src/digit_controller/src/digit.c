#include "digit.h"

#include <zephyr/logging/log.h>

#define UPDATE_MASK_ALL 0x7F
#define STATE_INITIALIZED_BIT 0x80

LOG_MODULE_DECLARE(digit_controller, LOG_LEVEL_DBG);

int digit_init(struct digit_ctx_t *ctx) {
    ctx->state &= ~STATE_INITIALIZED_BIT;
    return 0;
}

int digit_set_segments(struct digit_ctx_t *ctx, uint8_t new_state) {
    uint8_t update_mask = UPDATE_MASK_ALL;
    uint8_t old_state = ctx->state & ~STATE_INITIALIZED_BIT;

    if ((new_state & STATE_INITIALIZED_BIT) > 0) {
        return -EINVAL;
    }

    if ((ctx->state & STATE_INITIALIZED_BIT) > 0) {
        update_mask = old_state ^ new_state;
    }

    printk("Updating state from 0x%02x to 0x%02x, mask: 0x%02x\n",
            old_state,
            new_state,
            update_mask);

    ctx->state = new_state | STATE_INITIALIZED_BIT;
    return 0;
}

int digit_set_number(struct digit_ctx_t *ctx, uint8_t number) {
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
        return -EINVAL;
    }

    return digit_set_segments(ctx, states[number]);
}

int digit_clear(struct digit_ctx_t *ctx) {
    return digit_set_segments(ctx, 0x00);
}
