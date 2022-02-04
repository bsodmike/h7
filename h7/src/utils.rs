use {
    core::cell::RefCell,
    cortex_m::interrupt::{CriticalSection, Mutex},
    stm32h7xx_hal::crc::{Config, Crc},
};

pub static CRC: Mutex<RefCell<Option<Crc>>> = Mutex::new(RefCell::new(None));

pub fn crc(cs: &CriticalSection, data: &[u8]) -> u32 {
    match *CRC.borrow(cs).borrow_mut() {
        Some(ref mut crc) => {
            //
            let config = Config::new();
            crc.set_config(&config);
            crc.update_and_read(data)
        }
        None => 0,
    }
}
