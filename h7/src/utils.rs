use {
    crate::led::LED,
    core::cell::RefCell,
    cortex_m::interrupt::{self, CriticalSection, Mutex},
    stm32h7xx_hal::crc::{Config, Crc},
};

pub static CRC: Mutex<RefCell<Option<Crc>>> = Mutex::new(RefCell::new(None));

pub fn crc(cs: &CriticalSection, data: &[u8]) -> u32 {
    match *CRC.borrow(cs).borrow_mut() {
        Some(ref mut crc) => {
            let config = Config::new();
            crc.set_config(&config);
            crc.update_and_read(data)
        }
        None => 0,
    }
}

#[inline(always)]
pub fn interrupt_free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe { LED::Blue.on() };
    let r = interrupt::free(f);
    unsafe { LED::Blue.off() };
    r
}

#[inline(always)]
pub fn into_ok_or_err<T>(result: Result<T, T>) -> T {
    match result {
        Ok(v) => v,
        Err(v) => v,
    }
}
