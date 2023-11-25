#[cfg(debug_assertions)]
static mut LOG_LEVEL: log::LevelFilter = log::LevelFilter::Trace;

#[cfg(not(debug_assertions))]
static mut LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

pub fn get_log_level() -> log::LevelFilter {
    unsafe { LOG_LEVEL }
}

#[cfg(feature = "semihosting")]
mod semihosting {
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
            level: log::LevelFilter::Trace,
            inner: semihosting::InterruptOk::<_>::stdout().expect("Get Semihosting stdout"),
        };
    }

    pub fn init() {
        cortex_m_log::log::init(&LOGGER).unwrap();
    }

    pub fn set_log_level(_: log::LevelFilter) {}
}

#[cfg(feature = "semihosting")]
pub use semihosting::{init, set_log_level};

#[cfg(not(feature = "semihosting"))]
mod uart {

    use {
        crate::{terminal::UART_TERMINAL_TX, utils::interrupt_free},
        core::fmt::Write,
        log::{Log, Metadata, Record},
    };

    static LOGGER: UartLogger = UartLogger;

    pub struct UartLogger;

    impl Write for UartLogger {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            interrupt_free(|cs| {
                if let Some(tx) = &mut *UART_TERMINAL_TX.borrow(cs).borrow_mut() {
                    tx.write_str(s)?;
                }
                Ok(())
            })
        }
    }

    impl Log for UartLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= unsafe { super::LOG_LEVEL }
        }

        fn log(&self, record: &Record) {
            // haha rust go brrr
            // let this = self as *const Self as *mut Self;
            // let this = unsafe { &mut *this };
            let this = &mut UartLogger;
            let _ = write!(
                this,
                "{}",
                format_args!(
                    "[{level}] {file}:{line}: {msg}\n",
                    level = record.level(),
                    file = record.file().unwrap_or("<unknown>"),
                    line = record.line().unwrap_or(0),
                    msg = record.args()
                )
            );
        }

        fn flush(&self) {}
    }

    pub fn init() {
        set_log_level(unsafe { super::LOG_LEVEL });
        unsafe { log::set_logger_racy(&LOGGER) }.unwrap();
    }

    pub fn set_log_level(level: log::LevelFilter) {
        unsafe { super::LOG_LEVEL = level };
        log::set_max_level(level);
    }
}

#[cfg(not(feature = "semihosting"))]
pub use uart::{init, set_log_level};
