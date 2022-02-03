use {
    crate::{consts, time::TimeSource},
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    core::{cell::RefCell, fmt::Write},
    cortex_m::interrupt::{free as interrupt_free, Mutex},
    hds::Queue,
    menu::{check_args_len, MenuError, MenuItem},
    stm32h7xx_hal::{self as hal, interrupt, pac, prelude::*, serial},
};

pub struct TerminalWriter;

impl core::fmt::Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt_free(|cs| {
            if let Some(tx) = &mut *UART_TERMINAL_TX.borrow(cs).borrow_mut() {
                write!(tx, "{}", s)?
            }
            Ok(())
        })
    }
}

const HEADER_WIDTH: usize = 48;
const LABEL_WIDTH: usize = 20;
const DATE_PARSE_FORMAT: &str = "%Y-%m-%d";
const TIME_PARSE_FORMAT: &str = "%H:%M:%S";

// Terminal
pub static TERMINAL_INPUT_FIFO: Mutex<RefCell<Queue<u8, 64>>> =
    Mutex::new(RefCell::new(Queue::new()));
pub static UART_TERMINAL_RX: Mutex<RefCell<Option<serial::Rx<pac::USART1>>>> =
    Mutex::new(RefCell::new(None));
pub static UART_TERMINAL_TX: Mutex<RefCell<Option<serial::Tx<pac::USART1>>>> =
    Mutex::new(RefCell::new(None));
pub const UART_TERMINAL_BAUD: u32 = 115_200;

pub const MENU: &[MenuItem<TerminalWriter>] = &[
    MenuItem::Command {
        name: "help",
        help: "help <program> - Show help about a program",
        description: "Show help about a program",
        action: |m, args| {
            check_args_len(1, args.len())?;
            let program = args[0];
            for item in MENU {
                match item {
                    MenuItem::Command { name, help, .. } => {
                        if *name == program {
                            writeln!(m.writer(), "{}", help)?;
                            return Ok(());
                        }
                    }
                    MenuItem::Alias { alias, command } => {
                        if *alias == program {
                            writeln!(m.writer(), "'{}' aliased to '{}'", alias, command)?;
                            m.run("help", &[*command])?;
                            return Ok(());
                        }
                    }
                }
            }
            Err(MenuError::CommandNotFound)
        },
    },
    MenuItem::Command {
        name: "programs",
        help: "programs - Show available builtin programs",
        description: "Show available builtin programs",
        action: |m, args| {
            check_args_len(0, args.len())?;
            for item in MENU {
                match item {
                    MenuItem::Command {
                        name, description, ..
                    } => {
                        writeln!(
                            m.writer(),
                            "{:width$} {}",
                            name,
                            description,
                            width = LABEL_WIDTH
                        )?;
                    }
                    MenuItem::Alias { alias, command } => {
                        writeln!(
                            m.writer(),
                            "{:width$} aliased to {}",
                            alias,
                            command,
                            width = LABEL_WIDTH
                        )?;
                    }
                }
            }
            Ok(())
        },
    },
    MenuItem::Command {
        name: "pload",
        help: "pload <path/to/bin.h7> - Load a program into ram",
        description: "Load a program into ram",
        action: |_, args| {
            check_args_len(1, args.len())?;
            Err(MenuError::CommandNotImplemented)
        },
    },
    MenuItem::Command {
        name: "prun",
        help: "prun - Run program loaded in ram",
        description: "Run program loaded in ram",
        action: |_, args| {
            check_args_len(0, args.len())?;
            Err(MenuError::CommandNotImplemented)
        },
    },
    MenuItem::Command {
        name: "panic",
        help: "panic - Cause a panic",
        description: "Cause a panic",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(m.writer(), "Panicing!")?;
            panic!("Panic command")
        },
    },
    MenuItem::Command {
        name: "reset",
        help: "reset - Reset the device",
        description: "Reset the device",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(m.writer(), "Resetting!")?;
            cortex_m::peripheral::SCB::sys_reset()
        },
    },
    MenuItem::Command {
        name: "date",
        help: "date - Get system date and time",
        description: "Get system date and time",
        action: |m, args| {
            match args {
                ["set", date, time] => match (
                    NaiveDate::parse_from_str(date, DATE_PARSE_FORMAT),
                    NaiveTime::parse_from_str(time, TIME_PARSE_FORMAT),
                ) {
                    (Ok(date), Ok(time)) => {
                        writeln!(m.writer(), "Setting new date and time")?;
                        match TimeSource::set_date_time(NaiveDateTime::new(date, time)) {
                            Ok(_) => {
                                write!(m.writer(), "New date and time: ")?;
                                m.run("date", &[])?;
                                Ok(())
                            }
                            Err(_) => writeln!(m.writer(), "Error: RTC not initialized"),
                        }
                    }
                    (Err(_), _) => writeln!(m.writer(), "Date parsing failed"),
                    (_, Err(_)) => writeln!(m.writer(), "Time parsing failed"),
                },
                ["set", date_or_time] => {
                    if let Ok(date) = NaiveDate::parse_from_str(date_or_time, DATE_PARSE_FORMAT) {
                        match TimeSource::set_date(date) {
                            Ok(_) => {
                                write!(m.writer(), "New date and time: ")?;
                                m.run("date", &[])?;
                                Ok(())
                            }
                            Err(_) => writeln!(m.writer(), "Error: RTC not initialized"),
                        }
                    } else if let Ok(time) =
                        NaiveTime::parse_from_str(date_or_time, TIME_PARSE_FORMAT)
                    {
                        match TimeSource::set_time(time) {
                            Ok(_) => {
                                write!(m.writer(), "New date and time: ")?;
                                m.run("date", &[])?;
                                Ok(())
                            }
                            Err(_) => writeln!(m.writer(), "Error: RTC not initialized"),
                        }
                    } else {
                        writeln!(m.writer(), "Invalid date or time '{}'", date_or_time)
                    }
                }
                ["set"] => {
                    writeln!(m.writer(), "Set new system date and time:")?;
                    writeln!(
                        m.writer(),
                        "Set date and time: date set {} {}",
                        DATE_PARSE_FORMAT,
                        TIME_PARSE_FORMAT
                    )?;
                    writeln!(m.writer(), "Set date: date set {}", DATE_PARSE_FORMAT)?;
                    writeln!(m.writer(), "Set time: date set {}", TIME_PARSE_FORMAT)
                }
                [] => match TimeSource::get_date_time() {
                    Some(dt) => writeln!(
                        m.writer(),
                        "{weekday} {month} {day} {hh:02}:{mm:02}:{ss:02} {year}",
                        weekday = dt.weekday(),
                        month = month_to_str(dt.month()),
                        day = dt.day(),
                        hh = dt.hour(),
                        mm = dt.minute(),
                        ss = dt.second(),
                        year = dt.year()
                    ),
                    None => writeln!(m.writer(), "Error: RTC not initialized"),
                },
                _ => writeln!(m.writer(), "Invalid usage"),
            }?;
            // check_args_len(0, args.len())?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "info",
        help: "info - Query information from the system",
        description: "Query information from the system",
        action: |m, args| match args {
            ["mcu"] => {
                writeln!(
                    m.writer(),
                    "{:width$} STM32H747",
                    "MCU",
                    width = LABEL_WIDTH
                )?;
                // Uid is 12 byts, hex string will be 24.
                let (id, id_len) = to_hex::<24>(hal::signature::Uid::read(), false);
                // SAFETY: to_hex always returns valid hex
                let id_str = unsafe { core::str::from_utf8_unchecked(&id[0..id_len]) };
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Unique ID",
                    id_str,
                    width = LABEL_WIDTH
                )?;
                Ok(())
            }
            ["cpu"] => {
                writeln!(
                    m.writer(),
                    "{:width$} Cortex-M7F",
                    "Core",
                    width = LABEL_WIDTH
                )?;
                match interrupt_free(crate::system::cpu_freq) {
                    Some(freq) => writeln!(
                        m.writer(),
                        "{:width$} {}MHz",
                        "Core frequency",
                        freq.0 / 1_000_000,
                        width = LABEL_WIDTH
                    )?,
                    None => writeln!(
                        m.writer(),
                        "{:width$} unavailable",
                        "Core frequency",
                        width = LABEL_WIDTH
                    )?,
                }
                match interrupt_free(crate::system::cpu_temp) {
                    Some(temp) => writeln!(
                        m.writer(),
                        "{:width$} {:.01}°C",
                        "Core temperature",
                        temp,
                        width = LABEL_WIDTH
                    )?,
                    None => writeln!(
                        m.writer(),
                        "{:width$} unavailable",
                        "Core temperature",
                        width = LABEL_WIDTH
                    )?,
                }
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Cycle count",
                    cortex_m::peripheral::DWT::cycle_count(),
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Instruction cache",
                    bool_to_enabled_disabled_str(cortex_m::peripheral::SCB::icache_enabled()),
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Data cache",
                    bool_to_enabled_disabled_str(cortex_m::peripheral::SCB::dcache_enabled()),
                    width = LABEL_WIDTH
                )?;
                Ok(())
            }
            ["mem"] => {
                writeln!(
                    m.writer(),
                    "{:width$} {}KiB",
                    "Internal RAM",
                    crate::system::ram_size() / 1024,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}KiB",
                    "Internal FLASH",
                    crate::system::flash_size() / 1024,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}KiB",
                    "External SDRAM",
                    crate::mem::sdram::SDRAM_SIZE / 1024,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}KiB",
                    "External FLASH",
                    crate::mem::qspi_store::QSPI_FLASH_SIZE / 1024,
                    width = LABEL_WIDTH
                )?;
                Ok(())
            }
            ["os"] => {
                writeln!(
                    m.writer(),
                    "{:width$} {used}b/{total}b ({fraction:.02}%)",
                    "Memory usage",
                    used = crate::ALLOCATOR.used(),
                    total = crate::mem::sdram::SDRAM_SIZE,
                    fraction = (crate::ALLOCATOR.used() as f64
                        / crate::mem::sdram::SDRAM_SIZE as f64)
                        * 100.0,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Rust version",
                    consts::RUSTC_VERSION,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Version",
                    consts::GIT_DESCRIBE,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Debug",
                    cfg!(debug_assertions),
                    width = LABEL_WIDTH
                )?;
                let dt = NaiveDate::from_ymd(
                    consts::COMPILE_TIME_YEAR,
                    consts::COMPILE_TIME_MONTH,
                    consts::COMPILE_TIME_DAY,
                )
                .and_hms(
                    consts::COMPILE_TIME_HOUR,
                    consts::COMPILE_TIME_MINUTE,
                    consts::COMPILE_TIME_SECOND,
                );
                writeln!(
                    m.writer(),
                    "{:width$} {weekday} {month} {day} {hh:02}:{mm:02}:{ss:02} {year}",
                    "Compiled",
                    weekday = dt.weekday(),
                    month = month_to_str(dt.month()),
                    day = dt.day(),
                    hh = dt.hour(),
                    mm = dt.minute(),
                    ss = dt.second(),
                    year = dt.year(),
                    width = LABEL_WIDTH
                )?;
                Ok(())
            }
            [] => {
                writeln!(m.writer(), "{:-^-width$}", " MCU ", width = HEADER_WIDTH)?;
                m.run("info", &["mcu"])?;
                writeln!(m.writer(), "{:-^-width$}", " CPU ", width = HEADER_WIDTH)?;
                m.run("info", &["cpu"])?;
                writeln!(m.writer(), "{:-^-width$}", " Memory ", width = HEADER_WIDTH)?;
                m.run("info", &["mem"])?;
                writeln!(
                    m.writer(),
                    "{:-^-width$}",
                    " SD Card ",
                    width = HEADER_WIDTH
                )?;
                m.run("sdcard", &["info"])?;
                writeln!(m.writer(), "{:-^-width$}", " OS ", width = HEADER_WIDTH)?;
                m.run("info", &["os"])?;
                writeln!(
                    m.writer(),
                    "{:-^-width$}",
                    " Date/Time ",
                    width = HEADER_WIDTH
                )?;
                m.run("date", args)?;
                Ok(())
            }
            _ => {
                writeln!(m.writer(), "Unknown query")?;
                Ok(())
            }
        },
    },
    MenuItem::Command {
        name: "sdcard",
        help: "sdcard <action> - Mount/Unmount SD Card",
        description: "Mount/Unmount SD Card",
        action: |m, args| {
            check_args_len(1, args.len())?;
            match args[0] {
                "i" | "info" => match interrupt_free(|cs| {
                    crate::mem::sdmmc_fs::SD_CARD
                        .borrow(cs)
                        .borrow_mut()
                        .as_mut()
                        .map(|sdfs| (sdfs.is_mounted(), sdfs.card_size()))
                }) {
                    Some((mounted, size)) => {
                        writeln!(
                            m.writer(),
                            "{:width$} {}",
                            "SD Card mounted",
                            mounted,
                            width = LABEL_WIDTH
                        )?;
                        match size {
                            Ok(bytes) => {
                                writeln!(
                                    m.writer(),
                                    "{:width$} {}GiB",
                                    "Size",
                                    bytes as f64 / (1024 * 1024 * 1024) as f64,
                                    width = LABEL_WIDTH
                                )?;
                                Ok(())
                            }
                            Err(e) => {
                                writeln!(
                                    m.writer(),
                                    "{:width$} {}",
                                    "Size",
                                    e,
                                    width = LABEL_WIDTH
                                )?;
                                Ok(())
                            }
                        }
                    }
                    None => {
                        writeln!(m.writer(), "SD Card controller not initialized")?;
                        Ok(())
                    }
                },
                "m" | "mount" => interrupt_free(|cs| {
                    match crate::mem::sdmmc_fs::SD_CARD
                        .borrow(cs)
                        .borrow_mut()
                        .as_mut()
                        // TODO: Inc to 50MHz?
                        .map(|sdfs| sdfs.mount::<hal::delay::Delay, _>(20.mhz(), 5, None))
                    {
                        Some(Ok(_)) => {
                            writeln!(m.writer(), "SD Card mounted")?;
                            Ok(())
                        }
                        Some(Err(e)) => {
                            writeln!(m.writer(), "{}", e)?;
                            Ok(())
                        }
                        None => {
                            writeln!(m.writer(), "SD Card controller not initialized")?;
                            Err(MenuError::CommandError(None))
                        }
                    }
                }),
                "u" | "unmount" => interrupt_free(|cs| {
                    match crate::mem::sdmmc_fs::SD_CARD
                        .borrow(cs)
                        .borrow_mut()
                        .as_mut()
                        .map(|sdfs| sdfs.unmount())
                    {
                        Some(Ok(_)) => {
                            writeln!(m.writer(), "SD Card unmounted")?;
                            Ok(())
                        }
                        Some(Err(e)) => {
                            writeln!(m.writer(), "{}", e)?;
                            Ok(())
                        }
                        None => {
                            writeln!(m.writer(), "SD Card controller not initialized")?;
                            Err(MenuError::CommandError(None))
                        }
                    }
                }),
                _ => {
                    writeln!(m.writer(), "Expected:")?;
                    writeln!(m.writer(), "\tm | mount - Mount SD Card")?;
                    writeln!(m.writer(), "\tu | unmount - Unmount SD Card")?;
                    writeln!(m.writer(), "\ti | info - SD Card Info")?;
                    Err(MenuError::InvalidArgument)
                }
            }
        },
    },
    MenuItem::Command {
        name: "testfn",
        help: "testnf",
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
    MenuItem::Alias {
        alias: "commands",
        command: "programs",
    },
];

fn month_to_str(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        n => unreachable!("Month {} does not exist", n),
    }
}

fn bool_to_enabled_disabled_str(b: bool) -> &'static str {
    match b {
        true => "enabled",
        false => "disabled",
    }
}

fn nibble_to_char(nibble: u8, lowercase: bool) -> Option<u8> {
    match nibble {
        0..=9 => Some(nibble + 48),
        10..=15 => Some(nibble + 55 + if lowercase { 32 } else { 0 }),
        _ => None,
    }
}

fn to_hex<const N: usize>(data: &[u8], lowercase: bool) -> ([u8; N], usize) {
    let mut res = [0u8; N];
    let len = data.len().min(N / 2);
    for (i, byte) in data.iter().enumerate() {
        let idx = i * 2;
        // These unwraps can never fail as long as nibble_to_char handles the tange 0..=15
        res[idx] = nibble_to_char(byte >> 4, lowercase).unwrap();
        res[idx + 1] = nibble_to_char(byte & 0x0f, lowercase).unwrap();
    }
    (res, len * 2)
}

#[interrupt]
fn USART1() {
    interrupt_free(|cs| {
        if let Some(uart) = &mut *UART_TERMINAL_RX.borrow(cs).borrow_mut() {
            if let Ok(w) = uart.read() {
                let _ = TERMINAL_INPUT_FIFO.borrow(cs).borrow_mut().push(w);
            }
        }
    });
}
