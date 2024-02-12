use {
    crate::{
        board::{set_blue_led, set_green_led, set_red_led, LedState},
        terminal,
        utils::interrupt_free,
        Led, LED_BLUE, LED_GREEN, LED_RED,
    },
    core::{borrow::BorrowMut, fmt::Write, panic::PanicInfo},
    stm32h7xx_hal::gpio::Output,
};

struct PanicLogger;

#[cfg(not(feature = "defmt-rtt"))]
impl Write for PanicLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt_free(|cs| {
            if let Some(tx) = &mut *terminal::UART_TERMINAL_TX.borrow(cs).borrow_mut() {
                write!(tx, "{s}")?
            }
            Ok(())
        })
    }
}

#[cfg(feature = "defmt-rtt")]
impl Write for PanicLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt_free(|cs| {
            defmt::info!("Panic: {}", s);

            Ok(())
        })
    }
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    // TODO: Render panic info to display

    let _ = writeln!(PanicLogger, "{panic_info}");
    const LIMIT: usize = 10_000_000;
    const LIMIT_DC: usize = LIMIT / 2;

    // stm32h7xx_hal::gpio::Pin<'I', 12, Output>

    unsafe {
        set_green_led(LedState::Off);
        set_blue_led(LedState::Off);

        loop {
            for i in 0..LIMIT {
                if i < LIMIT_DC {
                    set_red_led(LedState::On);
                } else {
                    set_red_led(LedState::Off);
                }
            }
        }
    };
}
