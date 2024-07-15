#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;
use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;
use crate::hal::i2c::Error;

enum SegmentState {
    Set,
    Cleared,
    Unknown,
}

struct Segment {
    state: SegmentState,
    pin_set: &dyn embedded_hal::digital::v2::OutputPin<Error = Error>,
    pin_clr: &dyn embedded_hal::digital::v2::OutputPin<Error = Error>,
}

impl Segment {
    fn show(&self, _enable: bool) {


    }

    fn clear_pins(&self) {
        self.pin_set.set_low().ok();
        self.pin_clr.set_low().ok();
    }
}

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        // (Re-)configure PA1 as output
        let mut led = cortex_m::interrupt::free(|cs| gpioa.pa15.into_push_pull_output(cs));

        loop {
            // Turn PA1 on a million times in a row
            for _ in 0..10_000 {
                led.set_high().ok();
            }
            // Then turn PA1 off a million times in a row
            for _ in 0..10_000 {
                led.set_low().ok();
            }
        }
    }

    loop {
        continue;
    }
}
