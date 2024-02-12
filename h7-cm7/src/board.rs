use crate::{utils::interrupt_free, LED_BLUE, LED_GREEN, LED_RED};

pub(crate) enum LedState {
    On,
    Off,
}

pub fn set_green_led(state: LedState) {
    match state {
        LedState::On => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_GREEN.borrow_ref_mut(cs) {
                pin.set_low()
            };
        }),
        LedState::Off => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_GREEN.borrow_ref_mut(cs) {
                pin.set_high()
            };
        }),
    };
}

pub fn set_blue_led(state: LedState) {
    match state {
        LedState::On => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_BLUE.borrow_ref_mut(cs) {
                pin.set_low()
            };
        }),
        LedState::Off => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_BLUE.borrow_ref_mut(cs) {
                pin.set_high()
            };
        }),
    };
}

pub fn set_red_led(state: LedState) {
    match state {
        LedState::On => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_RED.borrow_ref_mut(cs) {
                pin.set_low()
            };
        }),
        LedState::Off => interrupt_free(|cs| {
            if let Some(pin) = &mut *LED_RED.borrow_ref_mut(cs) {
                pin.set_high()
            };
        }),
    };
}
