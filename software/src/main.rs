#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;
use crate::hal::{delay::Delay, pac, prelude::*};
use cortex_m;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::{OutputPin, PinState};

const DIGIT_SELECTORS: usize = 3;
const DIGITS: usize = 0x01 << DIGIT_SELECTORS;
const SEGMENTS: usize = 7;

struct Segment<Gpio> {
    pin_set: Gpio,
    pin_clr: Gpio,
}

impl<
Gpio: OutputPin,
> Segment<Gpio> {
    fn new(set: Gpio, clear: Gpio) -> Self {
        Self { pin_set: set, pin_clr: clear }
    }

    fn set(&mut self, on: bool) {
        match on {
            true => {
                self.pin_set.set_high().ok();
                self.pin_clr.set_low().ok();
            }
            false => {
                self.pin_set.set_low().ok();
                self.pin_clr.set_high().ok();
            }
        }
    }

    fn clear_pins(&mut self) {
        self.pin_set.set_low().ok();
        self.pin_clr.set_low().ok();
    }
}

struct DigitController<Gpio> {
    //   *  A/0  *
    //  F/5     B/1
    //   *  G/6  *
    //  E/4     C/2
    //   *  D/3  *
    segments: [Segment<Gpio>; SEGMENTS],
    state: [Option<[bool; SEGMENTS]>; DIGITS],
}

impl<
Gpio: OutputPin,
> DigitController<Gpio> {
    fn new(a: Segment<Gpio>, b: Segment<Gpio>, c: Segment<Gpio>, d: Segment<Gpio>, e: Segment<Gpio>, f: Segment<Gpio>, g: Segment<Gpio>) -> Self {
        Self { segments: [a, b, c, d, e, f, g], state: [None; DIGITS] }
    }

    fn update(&mut self, digit: usize, new_state: [bool; SEGMENTS]) {
        if digit >= self.state.len() {
            return;
        }
        match self.state[digit] {
            None => {
                for segment in 0..self.segments.len() {
                    self.segments[segment].set(new_state[segment]);
                }
            }
            Some(old_state) => {
                for segment in 0..self.segments.len() {
                    // Energizing the coil if the state hasn't changed would be a waste of energy,
                    // only actuate segments when they are expected to change state.
                    if old_state[segment] != new_state[segment] {
                        self.segments[segment].set(new_state[segment]);
                    } else {
                        self.segments[segment].clear_pins();
                    }
                }
            }
        }
        self.state[digit] = Some(new_state);
    }

    fn display_number(&mut self, digit: usize, number: usize) {
        match number {
            0 => { self.update(digit, [true, true, true, true, true, true, false]); }
            1 => { self.update(digit, [false, true, true, false, false, false, false]); }
            2 => { self.update(digit, [true, true, false, true, true, false, true]); }
            3 => { self.update(digit, [true, true, true, true, false, false, true]); }
            4 => { self.update(digit, [false, true, true, false, false, true, true]); }
            5 => { self.update(digit, [true, false, true, true, false, true, true]); }
            6 => { self.update(digit, [true, false, true, true, true, true, true]); }
            7 => { self.update(digit, [true, true, true, false, false, false, false]); }
            8 => { self.update(digit, [true, true, true, true, true, true, true]); }
            9 => { self.update(digit, [true, true, true, true, false, true, true]); }
            _ => {}
        }
    }

    fn clear_pins(&mut self) {
        for segment in self.segments.iter_mut() {
            segment.clear_pins();
        }
    }
}

struct DigitSelector<Gpio> {
    pins_control: [Gpio; DIGIT_SELECTORS],
    pin_enable: Gpio,
}

impl<
Gpio: OutputPin,
> DigitSelector<Gpio> {
    fn new(control: [Gpio; DIGIT_SELECTORS], enable: Gpio) -> Self {
        Self { pins_control: control, pin_enable: enable }
    }

    fn strobe(&mut self, digit: usize, delay: &mut Delay) {
        // We can control a maximum of two to the power of "number of control pins" digits
        if digit >= (0x01 << self.pins_control.len()) {
            return;
        }
        for n in 0..self.pins_control.len() {
            let pin_state = if (digit & 0x01 << n) > 0 {PinState::High} else {PinState::Low};
            self.pins_control[n].set_state(pin_state).ok();
        }
        self.pin_enable.set_high().ok();
        delay.delay_ms(100_u16);
        self.pin_enable.set_low().ok();

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

    let (mut controller, mut selector) = cortex_m::interrupt::free(|cs| {
        (
            DigitController::new(
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
            ),
            DigitSelector::new(
                [
                    gpiob.pb5.into_push_pull_output(cs).downgrade(),
                    gpiob.pb4.into_push_pull_output(cs).downgrade(),
                    gpiob.pb3.into_push_pull_output(cs).downgrade(),
                ],
                gpioa.pa15.into_push_pull_output(cs).downgrade(),
            )
        )
    });

    loop {
        for number in 0..10 {
            for digit in 0..8 {
                controller.display_number(digit, (digit+number) % 10);
                selector.strobe(digit, &mut delay);
            }
            controller.clear_pins();
            delay.delay_ms(1000_u16);
        }
    }
}
