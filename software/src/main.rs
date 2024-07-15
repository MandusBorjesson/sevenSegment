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

type SegmentPins<Set, Clear> = (Clear, Set);

struct Segment<SetPin: OutputPin, ClearPin: OutputPin> {
    state: SegmentState,
    pins: SegmentPins<SetPin, ClearPin>,
}

impl<
SetPin: OutputPin,
ClearPin: OutputPin,
> Segment<SetPin, ClearPin> {
    fn new(pins: SegmentPins<SetPin, ClearPin>) -> Self {
        Self { state: SegmentState::Unknown, pins }
    }

    fn set(&mut self, on: bool) {
        let new_state = if on {SegmentState::On} else {SegmentState::Off};

        // Energizing the coil at this point would only be a waste of energy, only actuate them
        // when they are expected to change state.
        if self.state == new_state {
            return;
        }

        match new_state {
            SegmentState::On => { self.pins.1.set_high().ok(); }
            SegmentState::Off => { self.pins.0.set_high().ok(); }
            _ => {}
        }
        self.state = new_state;
    }
    fn clear_pins(&mut self) {
        self.pins.1.set_low().ok();
        self.pins.0.set_low().ok();
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
    let pins = cortex_m::interrupt::free(|cs| {
        (gpioa.pa0.into_push_pull_output(cs),
        gpioa.pa7.into_push_pull_output(cs),
        )
    });

    let mut segment = Segment::new(pins);
    let mut on = false;

    loop {
        on = !on;
        segment.set(on);
        delay.delay_ms(100_u16);

        segment.clear_pins();
        delay.delay_ms(1000_u16);
    }
}
