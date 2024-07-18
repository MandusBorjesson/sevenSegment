#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;
use stm32f0xx_hal::gpio::{Output, Pin, PushPull};
use crate::hal::{delay::Delay, pac, prelude::*};
use cortex_m;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::PinState;

const DIGIT_SELECTORS: usize = 3;
const DIGITS: usize = 0x01 << DIGIT_SELECTORS;
const SEGMENTS: usize = 7;

#[derive(PartialEq)]
enum HwError {
    OutOfRange,
    DoesNotExist,
}

struct Segment {
    pin_set: Pin<Output<PushPull>>,
    pin_clr: Pin<Output<PushPull>>,
}

impl Segment {
    fn new(set: Pin<Output<PushPull>>, clear: Pin<Output<PushPull>>) -> Self {
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

struct DigitController {
    //   *  A/0  *
    //  F/5     B/1
    //   *  G/6  *
    //  E/4     C/2
    //   *  D/3  *
    segments: [Segment; SEGMENTS],
    state: [Option<[bool; SEGMENTS]>; DIGITS],
}

impl DigitController {
    fn new(a: Segment, b: Segment, c: Segment, d: Segment, e: Segment, f: Segment, g: Segment) -> Self {
        Self { segments: [a, b, c, d, e, f, g], state: [None; DIGITS] }
    }

    fn update(&mut self, digit: usize, new_state: [bool; SEGMENTS]) -> Result<bool, HwError> {
        if digit >= self.state.len() {
            return Err(HwError::DoesNotExist);
        }
        let mut requires_update: bool = false;

        match self.state[digit] {
            None => {
                for segment in 0..self.segments.len() {
                    self.segments[segment].set(new_state[segment]);
                    requires_update = true;
                }
            }
            Some(old_state) => {
                for segment in 0..self.segments.len() {
                    // Energizing the coil if the state hasn't changed would be a waste of energy,
                    // only actuate segments when they are expected to change state.
                    if old_state[segment] != new_state[segment] {
                        self.segments[segment].set(new_state[segment]);
                        requires_update = true;
                    } else {
                        self.segments[segment].clear_pins();
                    }
                }
            }
        }
        self.state[digit] = Some(new_state);
        Ok(requires_update)
    }

    fn display_number(&mut self, digit: usize, number: Option<usize>) -> Result<bool, HwError> {
        match number {
            Some(0) => { self.update(digit, [true, true, true, true, true, true, false]) }
            Some(1) => { self.update(digit, [false, true, true, false, false, false, false]) }
            Some(2) => { self.update(digit, [true, true, false, true, true, false, true]) }
            Some(3) => { self.update(digit, [true, true, true, true, false, false, true]) }
            Some(4) => { self.update(digit, [false, true, true, false, false, true, true]) }
            Some(5) => { self.update(digit, [true, false, true, true, false, true, true]) }
            Some(6) => { self.update(digit, [true, false, true, true, true, true, true]) }
            Some(7) => { self.update(digit, [true, true, true, false, false, false, false]) }
            Some(8) => { self.update(digit, [true, true, true, true, true, true, true]) }
            Some(9) => { self.update(digit, [true, true, true, true, false, true, true]) }
            None => { self.update(digit, [false, false, false, false, false, false, false]) }
            _ => { Err(HwError::OutOfRange) }
        }
    }

    fn clear_pins(&mut self) {
        for segment in self.segments.iter_mut() {
            segment.clear_pins();
        }
    }
}

struct DigitSelector {
    pins_control: [Pin<Output<PushPull>>; DIGIT_SELECTORS],
    pin_enable: Pin<Output<PushPull>>,
}

impl DigitSelector {
    fn new(control: [Pin<Output<PushPull>>; DIGIT_SELECTORS], enable: Pin<Output<PushPull>>) -> Self {
        Self { pins_control: control, pin_enable: enable }
    }

    fn strobe(&mut self, digit: usize, delay: &mut Delay)  -> Result<(), HwError> {
        // We can control a maximum of two to the power of "number of control pins" digits
        if digit >= (0x01 << self.pins_control.len()) {
            return Err(HwError::DoesNotExist);
        }
        for n in 0..self.pins_control.len() {
            let pin_state = if (digit & 0x01 << n) > 0 {PinState::High} else {PinState::Low};
            self.pins_control[n].set_state(pin_state).ok();
        }
        self.pin_enable.set_high().ok();
        delay.delay_ms(100_u16);
        self.pin_enable.set_low().ok();
        Ok(())
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

    let mut count = 0;
    loop {
        let mut tmp = count;
        for digit in 0..DIGITS {
            // Don't show leading zeros. Instead, blank the digits
            let digit_val = if tmp > 0 || (count == 0 && digit == 0) {Some(tmp % 10)} else {None};
            tmp /= 10;

            let result = controller.display_number(digit, digit_val);
            if result == Ok(true) {
                // We can't do much but cry if this fails, ignore the result...
                let _ = selector.strobe(digit, &mut delay);
            }
        }
        controller.clear_pins();
        delay.delay_ms(1000_u16);
        count += 1;
    }
}
