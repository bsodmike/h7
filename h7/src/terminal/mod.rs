use {
    crate::utils::interrupt_free,
    core::{cell::RefCell, fmt::Write},
    critical_section::Mutex,
    heapless::mpmc::Q64,
    menu::MenuItem,
    stm32h7xx_hal::{interrupt, pac, prelude::*, serial},
};

mod commands;
pub mod menu;

pub struct TerminalWriter;

impl core::fmt::Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt_free(|cs| {
            if let Some(tx) = &mut *UART_TERMINAL_TX.borrow(cs).borrow_mut() {
                write!(tx, "{s}")?
            }
            Ok(())
        })
    }
}

// Terminal
// pub static TERMINAL_INPUT_FIFO: Mutex<RefCell<Queue<u8, 64>>> =
//     Mutex::new(RefCell::new(Queue::new()));
pub static TERMINAL_INPUT_FIFO: Q64<u8> = Q64::new();
pub static UART_TERMINAL_RX: Mutex<RefCell<Option<serial::Rx<pac::USART1>>>> =
    Mutex::new(RefCell::new(None));
pub static UART_TERMINAL_TX: Mutex<RefCell<Option<serial::Tx<pac::USART1>>>> =
    Mutex::new(RefCell::new(None));
pub const UART_TERMINAL_BAUD: u32 = 115_200;

pub const MENU: &[MenuItem<TerminalWriter>] = &[
    commands::fs::LS,
    commands::fs::SDCARD,
    commands::program::PLOAD,
    commands::program::PRUN,
    commands::program::UPLOAD,
    commands::sys::COMMANDS,
    commands::sys::HELP,
    commands::sys::INFO,
    commands::sys::PROGRAMS,
    commands::sys::SYS,
    commands::time::CAL,
    commands::time::DATE,
    commands::time::TIME,
    MenuItem::Command {
        name: "testfn",
        help: "testfn",
        description: "testfn",
        action: |m, _| {
            writeln!(m.writer(), "testfn")?;
            // writeln!(m.writer(), "u64", mem::align_of::<64>())?;
            // mem!(m, align_of, u32)?;
            // mem!(m, align_of, &[u32])?;
            // mem!(m, align_of, u64)?;
            // mem!(m, align_of, &[u64])?;
            // mem!(m, align_of, u128)?;
            // mem!(m, align_of, &[u128])?;
            Ok(())
        },
    },
];

#[interrupt]
fn USART1() {
    interrupt_free(|cs| {
        if let Some(uart) = &mut *UART_TERMINAL_RX.borrow(cs).borrow_mut() {
            if let Ok(w) = uart.read() {
                let _ = TERMINAL_INPUT_FIFO.enqueue(w);
            }
        }
    });
}
