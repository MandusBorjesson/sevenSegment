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

//  * A *
//  F   B
//  * G *
//  E   C
//  * D *

struct Digits<Gpio> {
    a: Segment<Gpio>,
    b: Segment<Gpio>,
    c: Segment<Gpio>,
    d: Segment<Gpio>,
    e: Segment<Gpio>,
    f: Segment<Gpio>,
    g: Segment<Gpio>,
}

impl<
Gpio: OutputPin,
> Digits<Gpio> {
    fn new(a: Segment<Gpio>, b: Segment<Gpio>, c: Segment<Gpio>, d: Segment<Gpio>, e: Segment<Gpio>, f: Segment<Gpio>, g: Segment<Gpio>) -> Self {
        Self { a: a, b: b, c: c, d: d, e: e, f: f, g: g }
    }

    fn display(&mut self, digit: u16) {
        match digit {
            0 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(true);
                self.d.set(true);
                self.e.set(true);
                self.f.set(true);
                self.g.set(false);
            }
            1 => {
                self.a.set(false);
                self.b.set(true);
                self.c.set(true);
                self.d.set(false);
                self.e.set(false);
                self.f.set(false);
                self.g.set(false);
            }
            2 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(false);
                self.d.set(true);
                self.e.set(true);
                self.f.set(false);
                self.g.set(true);
            }
            3 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(true);
                self.d.set(true);
                self.e.set(false);
                self.f.set(false);
                self.g.set(true);
            }
            4 => {
                self.a.set(false);
                self.b.set(true);
                self.c.set(true);
                self.d.set(false);
                self.e.set(false);
                self.f.set(true);
                self.g.set(true);
            }
            5 => {
                self.a.set(true);
                self.b.set(false);
                self.c.set(true);
                self.d.set(true);
                self.e.set(false);
                self.f.set(true);
                self.g.set(true);
            }
            6 => {
                self.a.set(true);
                self.b.set(false);
                self.c.set(true);
                self.d.set(true);
                self.e.set(true);
                self.f.set(true);
                self.g.set(true);
            }
            7 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(true);
                self.d.set(false);
                self.e.set(false);
                self.f.set(false);
                self.g.set(false);
            }
            8 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(true);
                self.d.set(true);
                self.e.set(true);
                self.f.set(true);
                self.g.set(true);
            }
            9 => {
                self.a.set(true);
                self.b.set(true);
                self.c.set(true);
                self.d.set(true);
                self.e.set(false);
                self.f.set(true);
                self.g.set(true);
            }
            _ => {}
        }
    }

    fn clear_pins(&mut self) {
        self.a.clear_pins();
        self.b.clear_pins();
        self.c.clear_pins();
        self.d.clear_pins();
        self.e.clear_pins();
        self.f.clear_pins();
        self.g.clear_pins();
    }
}
#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
    let mut delay = Delay::new(cp.SYST, &rcc);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);

    let mut digits = cortex_m::interrupt::free(|cs| {
        Digits::new(
            // A
            Segment::new(
                gpioa.pa0.into_push_pull_output(cs).downgrade(),
                gpioa.pa7.into_push_pull_output(cs).downgrade(),
            ),
            // B
            Segment::new(
                gpioa.pa1.into_push_pull_output(cs).downgrade(),
                gpiob.pb0.into_push_pull_output(cs).downgrade(),
            ),
            // C
            Segment::new(
                gpioa.pa2.into_push_pull_output(cs).downgrade(),
                gpiob.pb1.into_push_pull_output(cs).downgrade(),
            ),
            // D
            Segment::new(
                gpioa.pa3.into_push_pull_output(cs).downgrade(),
                gpiob.pb2.into_push_pull_output(cs).downgrade(),
            ),
            // E
            Segment::new(
                gpioa.pa4.into_push_pull_output(cs).downgrade(),
                gpioa.pa8.into_push_pull_output(cs).downgrade(),
            ),
            // F
            Segment::new(
                gpioa.pa5.into_push_pull_output(cs).downgrade(),
                gpioa.pa11.into_push_pull_output(cs).downgrade(),
            ),
            // G
            Segment::new(
                gpioa.pa6.into_push_pull_output(cs).downgrade(),
                gpioa.pa12.into_push_pull_output(cs).downgrade(),
            )
        )
    });

    loop {
        for n in 0..10 {
            digits.display(n);
            delay.delay_ms(100_u16);
            digits.clear_pins();
            delay.delay_ms(1000_u16);
        }
    }
}
