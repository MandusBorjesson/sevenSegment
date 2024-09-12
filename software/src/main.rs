#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;
use stm32f0xx_hal::gpio::{Output, Pin, PushPull};
use crate::hal::{delay::Delay, pac, prelude::*};
use cortex_m;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::PinState;

const DIGITS: usize = 4;
const SEGMENTS: usize = 7;

#[derive(PartialEq)]
enum HwError {
    OutOfRange,
    DoesNotExist,
}

trait DigitController {

    fn update(&mut self, digit: usize, new_state: [bool; SEGMENTS]) -> Result<bool, HwError>;
    fn clear_pins(&mut self);

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
}

struct DigitControllerBitBangedExpander {
    pin_data: Pin<Output<PushPull>>,
    pin_clock: Pin<Output<PushPull>>,
    pin_enable: Pin<Output<PushPull>>,
    state: [Option<[bool; SEGMENTS]>; DIGITS],
}

impl DigitControllerBitBangedExpander {
    fn new(data: Pin<Output<PushPull>>, clock: Pin<Output<PushPull>>, enable: Pin<Output<PushPull>>) -> Self {
        Self { pin_data: data, pin_clock: clock, pin_enable: enable, state: [None; DIGITS] }
    }
}

impl DigitController for DigitControllerBitBangedExpander {
    fn update(&mut self, digit: usize, new_state: [bool; SEGMENTS]) -> Result<bool, HwError> {
        if digit >= self.state.len() {
            return Err(HwError::DoesNotExist);
        }

        let mut reg_state: u16 = 0;

        const A_SET: u16 = 13;
        const A_CLR: u16 = 15;
        const B_SET: u16 = 9;
        const B_CLR: u16 = 11;
        const C_SET: u16 = 4;
        const C_CLR: u16 = 6;
        const D_SET: u16 = 2;
        const D_CLR: u16 = 1;
        const E_SET: u16 = 5;
        const E_CLR: u16 = 3;
        const F_SET: u16 = 10;
        const F_CLR: u16 = 7;
        const G_SET: u16 = 12;
        const G_CLR: u16 = 14;

        match self.state[digit] {
            None => {
                reg_state |= 1 << (if new_state[0] { A_SET } else { A_CLR });
                reg_state |= 1 << (if new_state[1] { B_SET } else { B_CLR });
                reg_state |= 1 << (if new_state[2] { C_SET } else { C_CLR });
                reg_state |= 1 << (if new_state[3] { D_SET } else { D_CLR });
                reg_state |= 1 << (if new_state[4] { E_SET } else { E_CLR });
                reg_state |= 1 << (if new_state[5] { F_SET } else { F_CLR });
                reg_state |= 1 << (if new_state[6] { G_SET } else { G_CLR });
            }
            Some(old_state) => {
                reg_state |= ((old_state[0] != new_state[0]) as u16) << (if new_state[0] { A_SET } else { A_CLR });
                reg_state |= ((old_state[1] != new_state[1]) as u16) << (if new_state[1] { B_SET } else { B_CLR });
                reg_state |= ((old_state[2] != new_state[2]) as u16) << (if new_state[2] { C_SET } else { C_CLR });
                reg_state |= ((old_state[3] != new_state[3]) as u16) << (if new_state[3] { D_SET } else { D_CLR });
                reg_state |= ((old_state[4] != new_state[4]) as u16) << (if new_state[4] { E_SET } else { E_CLR });
                reg_state |= ((old_state[5] != new_state[5]) as u16) << (if new_state[5] { F_SET } else { F_CLR });
                reg_state |= ((old_state[6] != new_state[6]) as u16) << (if new_state[6] { G_SET } else { G_CLR });
            }
        }
        let requires_update: bool = if reg_state != 0 { true } else { false };
        self.state[digit] = Some(new_state);
        Ok(requires_update)
    }

    fn clear_pins(&mut self) {
    }
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

struct DigitControllerDirect {
    //   *  A/0  *
    //  F/5     B/1
    //   *  G/6  *
    //  E/4     C/2
    //   *  D/3  *
    segments: [Segment; SEGMENTS],
    state: [Option<[bool; SEGMENTS]>; DIGITS],
}

impl DigitControllerDirect {
    fn new(a: Segment, b: Segment, c: Segment, d: Segment, e: Segment, f: Segment, g: Segment) -> Self {
        Self { segments: [a, b, c, d, e, f, g], state: [None; DIGITS] }
    }
}

impl DigitController for DigitControllerDirect {
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

    fn clear_pins(&mut self) {
        for segment in self.segments.iter_mut() {
            segment.clear_pins();
        }
    }
}

struct DigitSelector {
    pins_control: [Pin<Output<PushPull>>; DIGITS],
}

impl DigitSelector {
    fn new(control: [Pin<Output<PushPull>>; DIGITS]) -> Self {
        Self { pins_control: control }
    }

    fn strobe(&mut self, digit: usize, delay: &mut Delay)  -> Result<(), HwError> {
        if digit >= self.pins_control.len() {
            return Err(HwError::DoesNotExist);
        }
        self.pins_control[digit].set_high().ok();
        delay.delay_ms(100_u16);
        self.pins_control[digit].set_low().ok();
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
            DigitControllerBitBangedExpander::new(
                gpioa.pa7.into_push_pull_output(cs).downgrade(),
                gpioa.pa5.into_push_pull_output(cs).downgrade(),
                gpioa.pa4.into_push_pull_output(cs).downgrade(),
            ),
            DigitSelector::new(
                [
                    gpiob.pb0.into_push_pull_output(cs).downgrade(),
                    gpiob.pb1.into_push_pull_output(cs).downgrade(),
                    gpiob.pb2.into_push_pull_output(cs).downgrade(),
                    gpioa.pa8.into_push_pull_output(cs).downgrade(),
                ],
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
