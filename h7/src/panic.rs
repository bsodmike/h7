use {
    crate::{terminal, LED},
    core::{fmt::Write, panic::PanicInfo},
    cortex_m::interrupt::free as interrupt_free,
};

struct PanicLogger;

impl Write for PanicLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt_free(|cs| {
            if let Some(tx) = &mut *terminal::UART_TERMINAL_TX.borrow(cs).borrow_mut() {
                write!(tx, "{}", s)?
            }
            Ok(())
        })
    }
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    let _ = writeln!(PanicLogger, "{}", panic_info);
    const LIMIT: usize = 10_000_000;
    const LIMIT_DC: usize = LIMIT / 2;
    unsafe {
        LED::Green.off();
        LED::Blue.off();
        loop {
            for i in 0..LIMIT {
                if i < LIMIT_DC {
                    LED::Red.on()
                } else {
                    LED::Red.off()
                }
            }
        }
    };
}
