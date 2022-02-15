use {
    crate::{
        app, consts,
        menu::{check_args_len, MenuError, MenuItem},
        time::TimeSource,
        utils::interrupt_free,
    },
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    core::{cell::RefCell, fmt::Write},
    cortex_m::interrupt::Mutex,
    heapless::mpmc::Q64,
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
// pub static TERMINAL_INPUT_FIFO: Mutex<RefCell<Queue<u8, 64>>> =
//     Mutex::new(RefCell::new(Queue::new()));
pub static TERMINAL_INPUT_FIFO: Q64<u8> = Q64::new();
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
        action: |m, args| {
            check_args_len(1, args.len())?;
            let app_slice = app::app_slice();
            // .bss
            app_slice.fill(0);
            interrupt_free(|cs| {
                match crate::mem::sdmmc_fs::SD_CARD
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .map(|sdfs| sdfs.read_file(args[0], app_slice))
                {
                    Some(Ok(len)) => {
                        writeln!(m.writer(), "Program '{}' loaded ({} bytes)", args[0], len)?;
                        app::print_info(m.writer(), &app_slice[..len])?;
                        Ok(())
                    }
                    Some(Err(e)) => {
                        writeln!(m.writer(), "Error: {}", e)?;
                        Ok(())
                    }
                    None => {
                        writeln!(m.writer(), "Error: SD Card controller not initialized")?;
                        Ok(())
                    }
                }
            })
        },
    },
    MenuItem::Command {
        name: "prun",
        help: "prun - Run program loaded in ram",
        description: "Run program loaded in ram",
        action: |m, args| {
            check_args_len(0, args.len())?;
            let app_slice = app::app_slice();
            let app_fn = app::get_address(app_slice);
            if app::check_address(app_fn).is_err() {
                return Err(MenuError::CommandError(Some("Invalid app address")));
            }
            writeln!(m.writer(), "Executing from {:p}", app_fn)?;
            let ret = unsafe {
                // Disable cache
                let mut cp = cortex_m::Peripherals::steal();
                cp.SCB.disable_icache();
                cp.SCB.invalidate_icache();
                cp.SCB.disable_dcache(&mut cp.CPUID);
                cp.SCB.clean_dcache(&mut cp.CPUID);

                // Sync
                cortex_m::asm::dmb();
                cortex_m::asm::dsb();
                cortex_m::asm::isb();

                // Run
                let ret = app_fn(&app::API);

                // Enable cache
                cp.SCB.enable_icache();
                cp.SCB.enable_dcache(&mut cp.CPUID);

                ret
            };
            writeln!(
                m.writer(),
                "Exit: {} ({})",
                ret,
                if ret == 0 { "ok" } else { "error" }
            )?;
            match app::free_leaked() {
                0 => { /* App did not leak memory */ }
                n => writeln!(m.writer(), "App leaked {} bytes", n)?,
            }

            Ok(())
        },
    },
    MenuItem::Command {
        name: "sys",
        help: "sys - Test system functionality",
        description: "Test system functionality",
        action: |m, args| match args {
            ["panic"] => {
                writeln!(m.writer(), "Panicing!")?;
                panic!("User caused a panic")
            }
            ["bkpt"] => {
                writeln!(m.writer(), "Breakpoint!")?;
                cortex_m::asm::bkpt();
                Ok(())
            }
            ["udf"] => {
                writeln!(m.writer(), "Undefined instruction!")?;
                cortex_m::asm::udf();
            }
            ["reset"] => {
                writeln!(m.writer(), "Resetting!")?;
                cortex_m::peripheral::SCB::sys_reset()
            }
            _ => check_args_len(1, args.len()),
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
        name: "cal",
        help: "Show calendar",
        description: "Show calendar",
        action: |m, args| {
            check_args_len(0, args.len())?;
            match TimeSource::get_date_time() {
                Some(ref dt) => {
                    let top = dt.with_day0(0).unwrap();
                    let day = dt.day0();
                    let wd = top.weekday() as u32;
                    let week = top.iso_week().week() - 1;
                    let n_days = days_in_month(dt);
                    writeln!(
                        m.writer(),
                        "         {m} {y}",
                        m = month_to_str(dt.month()),
                        y = dt.year()
                    )?;
                    writeln!(m.writer(), "Wk | Mo Tu We Th Fr Sa Su")?;
                    write!(m.writer(), "{:2} |", week + 1)?;
                    for _ in 0..wd {
                        write!(m.writer(), "   ")?;
                    }
                    for i in 0..n_days {
                        let c = i + wd + 1;
                        if i == day {
                            write!(m.writer(), "[{:2}", i + 1)?;
                            if wd % 7 == 0 {
                                write!(m.writer(), "]")?;
                            }
                        } else if i == day + 1 && !(c % 7 == 1) {
                            write!(m.writer(), "]{:2}", i + 1)?;
                        } else {
                            write!(m.writer(), " {:2}", i + 1)?;
                        }
                        if c % 7 == 0 && i != n_days - 1 {
                            writeln!(m.writer())?;
                            write!(m.writer(), "{:2} |", 1 + ((week + (c / 7)) % 52))?;
                        }
                    }
                    writeln!(m.writer())
                }
                None => writeln!(m.writer(), "Error: RTC not initialized"),
            }?;
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
                    "{:width$} {used} / {total} bytes ({fraction:.02}%)",
                    "Memory usage",
                    used = crate::mem::ALLOCATOR.used(),
                    total = crate::mem::HEAP_SIZE,
                    fraction = (crate::mem::ALLOCATOR.used() as f64 / crate::mem::HEAP_SIZE as f64)
                        * 100.0,
                    width = LABEL_WIDTH
                )?;
                writeln!(
                    m.writer(),
                    "{:width$} {} bytes",
                    "GPU reserved",
                    crate::display::FRAME_BUF_SIZE,
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
        action: |m, args| match args {
            ["i" | "info"] => match interrupt_free(|cs| {
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
                            writeln!(m.writer(), "{:width$} {}", "Size", e, width = LABEL_WIDTH)?;
                            Ok(())
                        }
                    }
                }
                None => {
                    writeln!(m.writer(), "SD Card controller not initialized")?;
                    Ok(())
                }
            },
            ["m" | "mount"] => m.run("sdcard", &["mount", "400"]),
            ["m" | "mount", freq_str] => interrupt_free(|cs| {
                let freq = freq_str
                    .parse::<u32>()
                    .map_err(|_| MenuError::InvalidArgument)?;
                writeln!(m.writer(), "Attempting to mount SD Card @ {}kHz", freq)?;
                match crate::mem::sdmmc_fs::SD_CARD
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .map(|sdfs| sdfs.mount::<hal::delay::Delay, _>(freq.khz(), 10, None))
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
            ["u" | "unmount"] => interrupt_free(|cs| {
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
        },
    },
    MenuItem::Command {
        name: "ls",
        help: "List files",
        description: "List files",
        action: |m, _| {
            interrupt_free(|cs| {
                match crate::mem::sdmmc_fs::SD_CARD
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .map(|sdfs| {
                        sdfs.files(|e| {
                            // let _ = writeln!(m.writer(), "{:?}", e);
                            if !e.attributes.is_volume() && !e.attributes.is_hidden() {
                                // let kind = if e.attributes.is_directory() {
                                //     "D"
                                // } else {
                                //     "F"
                                // };
                                // let _ = writeln!(
                                //     m.writer(),
                                //     "{date} {kind} {size:size_width$} {name}",
                                //     date = e.mtime,
                                //     size = e.size,
                                //     name = e.name,
                                //     size_width = 5
                                // );
                                let _ = if e.attributes.is_directory() {
                                    writeln!(m.writer(), "{:13} {}  <DIR>", e.name, e.mtime)
                                } else {
                                    writeln!(
                                        m.writer(),
                                        "{:13} {}  {} bytes",
                                        e.name,
                                        e.mtime,
                                        e.size
                                    )
                                };
                            }
                        })
                    }) {
                    Some(Ok(_)) => Ok(()),
                    Some(Err(e)) => {
                        writeln!(m.writer(), "Error: {}", e)?;
                        Ok(())
                    }
                    None => {
                        writeln!(m.writer(), "Error: SD Card controller not initialized")?;
                        Ok(())
                    }
                }
            })
        },
    },
    MenuItem::Command {
        name: "upload",
        help: "Load program into RAM via serial. Data is sent in ascii hex.",
        description: "Load program into RAM via serial. Data is sent in ascii hex.",
        action: |m, args| {
            let mut n = 0usize;
            let app_slice = app::app_slice();
            app_slice.fill(0);
            match args {
                [bin] => match bin.as_bytes().chunks(2).try_for_each(|s| match s.len() {
                    1 => Err((s[0], None)),
                    2 => {
                        let b = from_hex(s[0], s[1]).ok_or((s[0], Some(s[1])))?;
                        app_slice[n] = b;
                        n += 1;
                        Ok(())
                    }
                    _ => unreachable!(),
                }) {
                    Ok(_) => {}
                    Err((a, None)) => {
                        writeln!(
                            m.writer(),
                            "Invalid byte: '0x{} None' (half a byte missing)",
                            a as char
                        )?;
                        return Err(MenuError::InvalidArgument);
                    }
                    Err((a, Some(b))) => {
                        writeln!(
                            m.writer(),
                            "Invalid byte: '0x{} 0x{}'",
                            a as char,
                            b as char
                        )?;
                        return Err(MenuError::InvalidArgument);
                    }
                },
                _ => {
                    writeln!(m.writer(), "Waiting for data...")?;
                    let mut byte = None::<u8>;
                    loop {
                        match (
                            byte,
                            //  interrupt_free(|cs| TERMINAL_INPUT_FIFO.borrow(cs).borrow_mut().pop()),
                            TERMINAL_INPUT_FIFO.dequeue(),
                        ) {
                            (b, Some(b'\n')) => {
                                if let Some(b) = b {
                                    writeln!(
                                        m.writer(),
                                        "Invalid byte: '0x{} None' (half a byte missing)",
                                        b as char
                                    )?;
                                    return Err(MenuError::InvalidArgument);
                                } else {
                                    break;
                                }
                            }
                            (Some(x), Some(y)) => match from_hex(x, y) {
                                Some(b) => {
                                    app_slice[n] = b;
                                    n += 1;
                                    byte = None;
                                }
                                None => {
                                    writeln!(
                                        m.writer(),
                                        "Invalid byte: '0x{} 0x{}'",
                                        x as char,
                                        y as char
                                    )?;
                                    return Err(MenuError::InvalidArgument);
                                }
                            },
                            (None, n) => byte = n,
                            _ => {}
                        }
                    }
                }
            }
            writeln!(m.writer(), "Read {} bytes", n)?;
            if n > 8 {
                app::print_info(m.writer(), &app_slice[..n])?;
            } else {
                writeln!(m.writer(), "Not enough data")?;
            }

            Ok(())
        },
    },
    MenuItem::Command {
        name: "time",
        help: "Measure execution time of a command",
        description: "Measure execution time of a command",
        action: |m, args| {
            let start = TimeSource::get_date_time();
            match args {
                [] => Err(MenuError::InvalidArgument),
                [cmd, rest @ ..] => m.run(cmd, rest),
            }?;
            match (start, TimeSource::get_date_time()) {
                (Some(start), Some(end)) => writeln!(m.writer(), "Execution took {}", end - start),
                (None, _) => writeln!(m.writer(), "Failed to get start time"),
                (_, None) => writeln!(m.writer(), "Failed to get end time"),
            }?;
            Ok(())
        },
    },
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

fn days_in_month<D: Datelike>(d: &D) -> u32 {
    let y = d.year();
    let m = d.month();
    if m == 12 {
        NaiveDate::from_ymd(y + 1, 1, 1)
    } else {
        NaiveDate::from_ymd(y, m + 1, 1)
    }
    .signed_duration_since(NaiveDate::from_ymd(y, m, 1))
    .num_days() as u32
}

fn bool_to_enabled_disabled_str(b: bool) -> &'static str {
    match b {
        true => "enabled",
        false => "disabled",
    }
}

const fn nibble_to_char(nibble: u8, lowercase: bool) -> Option<u8> {
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
        // SAFETY: These unwraps can never fail as long as nibble_to_char handles the tange 0..=15
        unsafe {
            res[idx] = nibble_to_char(byte >> 4, lowercase).unwrap_unchecked();
            res[idx + 1] = nibble_to_char(byte & 0x0f, lowercase).unwrap_unchecked();
        }
    }
    (res, len * 2)
}

const fn from_hex(nibble1: u8, nibble2: u8) -> Option<u8> {
    let a = nibble1 | 0b0010_0000;
    let b = nibble2 | 0b0010_0000;

    let n1 = match a {
        b'0'..=b'9' => a - 48,
        b'a'..=b'f' => a - 87,
        _ => return None,
    };

    let n2 = match b {
        b'0'..=b'9' => b - 48,
        b'a'..=b'f' => b - 87,
        _ => return None,
    };

    Some((n1 << 4) | (n2 & 0x0f))
}

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
