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
            level: log::LevelFilter::Info,
            inner: semihosting::InterruptOk::<_>::stdout().expect("Get Semihosting stdout"),
        };
    }

    pub fn init() {
        cortex_m_log::log::init(&LOGGER).unwrap();
    }
}

#[cfg(feature = "semihosting")]
pub use semihosting::init;

#[cfg(not(feature = "semihosting"))]
mod uart {

    use {
        crate::terminal::UART_TERMINAL_TX,
        core::fmt::Write,
        cortex_m::interrupt::free as interrupt_free,
        log::{LevelFilter, Log, Metadata, Record},
    };

    static LOGGER: UartLogger = UartLogger {
        level: LevelFilter::Trace,
    };

    pub struct UartLogger {
        level: LevelFilter,
    }

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
            metadata.level() <= self.level
        }

        fn log(&self, record: &Record) {
            // haha rust go brrr
            let this = self as *const Self as *mut Self;
            let this = unsafe { &mut *this };
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
        log::set_max_level(LOGGER.level);
        unsafe { log::set_logger_racy(&LOGGER) }.unwrap();
    }
}

#[cfg(not(feature = "semihosting"))]
pub use uart::init;
