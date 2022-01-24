use {
    core::cell::RefCell,
    cortex_m::interrupt::{CriticalSection, Mutex},
    stm32h7xx_hal::{
        self as hal,
        adc::{self, Adc},
        prelude::*,
        signature::{TS_CAL_110, TS_CAL_30},
        time::Hertz,
    },
};

// As per schematic
const VDDA: f64 = 3.100;

extern "C" {
    static _ram_start: u32;
    static _ram_end: u32;
}

pub static CLOCK_FREQ: Mutex<RefCell<Option<Hertz>>> = Mutex::new(RefCell::new(None));
pub static CORE_TEMP: Mutex<
    RefCell<Option<(Adc<hal::device::ADC3, adc::Enabled>, adc::Temperature)>>,
> = Mutex::new(RefCell::new(None));

pub fn cpu_freq(cs: &CriticalSection) -> Option<Hertz> {
    *CLOCK_FREQ.borrow(cs).borrow()
}

pub fn cpu_temp(cs: &CriticalSection) -> Option<f64> {
    CORE_TEMP
        .borrow(cs)
        .borrow_mut()
        .as_mut()
        .and_then(|(adc, channel)| adc.read(channel).ok())
        .map(|word: u32| {
            ((110.0 - 30.0) / (TS_CAL_110::read() - TS_CAL_30::read()) as f64)
                * ((word as f64 * (VDDA / 3.3)) - TS_CAL_30::read() as f64)
                + 30.0
        })
}

pub fn ram_size() -> usize {
    unsafe { (&_ram_end as *const u32) as usize - (&_ram_start as *const u32) as usize }
}

pub fn flash_size() -> usize {
    hal::signature::FlashSize::bytes()
}
