#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;
use crate::hal::{delay::Delay, pac, prelude::*};
use cortex_m;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

#[derive(PartialEq)]
enum SegmentState {
    On,
    Off,
    Unknown,
}

struct Segment<Gpio> {
    state: SegmentState,
    pin_set: Gpio,
    pin_clr: Gpio,
}

impl<
Gpio: OutputPin,
> Segment<Gpio> {
    fn new(set: Gpio, clear: Gpio) -> Self {
        Self { state: SegmentState::Unknown, pin_set: set, pin_clr: clear }
    }

    fn set(&mut self, on: bool) {
        let new_state = if on {SegmentState::On} else {SegmentState::Off};

        // Energizing the coil at this point would only be a waste of energy, only actuate them
        // when they are expected to change state.
        if self.state == new_state {
            return;
        }

        match new_state {
            SegmentState::On => { self.pin_set.set_high().ok(); }
            SegmentState::Off => { self.pin_clr.set_high().ok(); }
            _ => {}
        }
        self.state = new_state;
    }
    fn clear_pins(&mut self) {
        self.pin_set.set_low().ok();
        self.pin_clr.set_low().ok();
    }
}

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
    let mut delay = Delay::new(cp.SYST, &rcc);

    let gpioa = p.GPIOA.split(&mut rcc);

    // (Re-)configure PA1 as output
    let mut segment = cortex_m::interrupt::free(|cs| {
        Segment::new(gpioa.pa0.into_push_pull_output(cs).downgrade(),
        gpioa.pa7.into_push_pull_output(cs).downgrade(),
        )
    });

    let mut on = false;

    loop {
        on = !on;
        segment.set(on);
        delay.delay_ms(100_u16);

        segment.clear_pins();
        delay.delay_ms(1000_u16);
    }
}
