use {
    crate::{consts, time::TimeSource},
    chrono::{Datelike, NaiveDate, Timelike},
    core::{cell::RefCell, fmt::Write},
    cortex_m::interrupt::{free as interrupt_free, Mutex},
    menu::{check_args_len, MenuError, MenuItem},
    ringfifo::RingFiFo,
    stm32h7xx_hal::{self as hal, interrupt, pac, prelude::*, serial},
};

// pub struct TerminalWriter;

// impl core::fmt::Write for TerminalWriter {
//     fn write_str(&mut self, s: &str) -> core::fmt::Result {
//         interrupt_free(|cs| {
//             if let Some(tx) = &mut *TERMINAL_TX.borrow(cs).borrow_mut() {
//                 write!(tx, "{}", s)
//             } else {
//                 Err(core::fmt::Error)
//             }
//         })
//     }
// }

const HEADER_WIDTH: usize = 52;
const LABEL_WIDTH: usize = 27;

// Terminal
pub static TERMINAL_INPUT_FIFO: Mutex<RefCell<RingFiFo<u8, 52>>> =
    Mutex::new(RefCell::new(RingFiFo::new()));
pub static TERMINAL_RX: Mutex<RefCell<Option<serial::Rx<pac::USART1>>>> =
    Mutex::new(RefCell::new(None));
// pub static TERMINAL_TX: Mutex<RefCell<Option<serial::Tx<pac::USART1>>>> =
//     Mutex::new(RefCell::new(None));

pub const MENU: &[MenuItem<serial::Tx<pac::USART1>>] = &[
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
                            write!(m.writer(), "{} aliased to {}", alias, command)?;
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
            check_args_len(0, args.len())?;
            match TimeSource::get_date_time() {
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
            }?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "mcuinfo",
        help: "mcuinfo - Get MCU information",
        description: "Get MCU information",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "MCU",
                "STM32H747",
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
        },
    },
    MenuItem::Command {
        name: "cpuinfo",
        help: "cpuinfo - Get CPU information",
        description: "Get CPU information",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "Core",
                "Cortex-M7F",
                width = LABEL_WIDTH
            )?;
            match interrupt_free(|cs| crate::system::cpu_freq(cs)) {
                Some(freq) => writeln!(
                    m.writer(),
                    "{:width$} {}MHz",
                    "Core frequency",
                    freq.0 / 1_000_000,
                    width = LABEL_WIDTH
                )?,
                None => writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Core frequency",
                    "unavailable",
                    width = LABEL_WIDTH
                )?,
            }
            match interrupt_free(|cs| crate::system::cpu_temp(cs)) {
                Some(temp) => writeln!(
                    m.writer(),
                    "{:width$} {:.01}°C",
                    "Core temperature",
                    temp,
                    width = LABEL_WIDTH
                )?,
                None => writeln!(
                    m.writer(),
                    "{:width$} {}",
                    "Core temperature",
                    "unavailable",
                    width = LABEL_WIDTH
                )?,
            }
            writeln!(
                m.writer(),
                "{:width$} {}",
                "Cycle count",
                cortex_m::peripheral::DWT::get_cycle_count(),
                width = LABEL_WIDTH
            )?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "Instruction cache enabled",
                cortex_m::peripheral::SCB::icache_enabled(),
                width = LABEL_WIDTH
            )?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "Data cache enabled",
                cortex_m::peripheral::SCB::dcache_enabled(),
                width = LABEL_WIDTH
            )?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "meminfo",
        help: "meminfo - Get memory information",
        description: "Get memory information",
        action: |m, args| {
            check_args_len(0, args.len())?;
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
                crate::sdram::SDRAM_SIZE / 1024,
                width = LABEL_WIDTH
            )?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "External FLASH",
                "unavailable",
                width = LABEL_WIDTH
            )?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "sdcardinfo",
        help: "sdcardinfo - Get SD Card information",
        description: "Get SD Card information",
        action: |m, args| {
            check_args_len(0, args.len())?;
            match interrupt_free(|cs| {
                crate::sdmmc_fs::SD_CARD
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
                        Ok(bytes) => writeln!(
                            m.writer(),
                            "{:width$} {}GiB",
                            "Size",
                            bytes as f64 / (1024 * 1024 * 1024) as f64,
                            width = LABEL_WIDTH
                        ),
                        Err(e) => {
                            writeln!(m.writer(), "{:width$} {}", "Size", e, width = LABEL_WIDTH)
                        }
                    }
                }
                None => writeln!(m.writer(), "SD Card controller not initialized"),
            }?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "osinfo",
        help: "osinfo - Get os information",
        description: "Get os information",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(
                m.writer(),
                "{:width$} {}",
                "Version",
                consts::GIT_DESCRIBE,
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
        },
    },
    MenuItem::Command {
        name: "sysinfo",
        help: "sysinfo - Get system information",
        description: "Get system information",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(m.writer(), "{:-^-width$}", " MCU ", width = HEADER_WIDTH)?;
            m.run("mcuinfo", args)?;
            writeln!(m.writer(), "{:-^-width$}", " CPU ", width = HEADER_WIDTH)?;
            m.run("cpuinfo", args)?;
            writeln!(m.writer(), "{:-^-width$}", " Memory ", width = HEADER_WIDTH)?;
            m.run("meminfo", args)?;
            writeln!(
                m.writer(),
                "{:-^-width$}",
                " SD Card ",
                width = HEADER_WIDTH
            )?;
            m.run("sdcardinfo", args)?;
            writeln!(m.writer(), "{:-^-width$}", " OS ", width = HEADER_WIDTH)?;
            m.run("osinfo", args)?;
            writeln!(
                m.writer(),
                "{:-^-width$}",
                " Date/Time ",
                width = HEADER_WIDTH
            )?;
            m.run("date", args)?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "sdcard",
        help: "sdcard <action> - Mount/Unmount SD Card",
        description: "Mount/Unmount SD Card",
        action: |m, args| {
            check_args_len(1, args.len())?;
            match args[0] {
                "m" | "mount" => interrupt_free(|cs| {
                    match crate::sdmmc_fs::SD_CARD
                        .borrow(cs)
                        .borrow_mut()
                        .as_mut()
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
                    match crate::sdmmc_fs::SD_CARD
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
                    writeln!(m.writer(), "\tu | unmount - Unmount SD Card:")?;
                    Err(MenuError::InvalidArgument)
                }
            }
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
    for i in 0..len {
        let idx = i * 2;
        // These unwraps can never fail as long as nibble_to_char handles the tange 0..=15
        res[idx] = nibble_to_char(data[i] >> 4, lowercase).unwrap();
        res[idx + 1] = nibble_to_char(data[i] & 0x0f, lowercase).unwrap();
    }
    (res, len * 2)
}

#[interrupt]
fn USART1() {
    interrupt_free(|cs| {
        if let Some(uart) = &mut *TERMINAL_RX.borrow(cs).borrow_mut() {
            match uart.read() {
                Ok(w) => TERMINAL_INPUT_FIFO.borrow(cs).borrow_mut().push_back(w),
                Err(e) => {}
            }
        }
    });
    unsafe {
        (*stm32h7xx_hal::pac::GPIOK::ptr())
            .bsrr
            .write(|w| w.bs7().set_bit())
    }
}
