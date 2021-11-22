use {
    cortex_m_log::{
        log::Logger,
        modes::InterruptOk,
        printer::semihosting::{self, Semihosting},
    },
    cortex_m_semihosting::hio::HStdout,
    panic_semihosting as _,
};

lazy_static::lazy_static! {
    static ref LOGGER: Logger<Semihosting<InterruptOk, HStdout>> = Logger {
        level: log::LevelFilter::Info,
        inner: semihosting::InterruptOk::<_>::stdout().expect("Get Semihosting stdout"),
    };
}

pub fn init() {
    cortex_m_log::log::init(&LOGGER).unwrap();
}
