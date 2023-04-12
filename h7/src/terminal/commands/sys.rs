use {
    super::{utils::*, HEADER_WIDTH, LABEL_WIDTH},
    crate::{
        consts, logger,
        terminal::{
            menu::{MenuError, MenuItem},
            TerminalWriter, MENU,
        },
        utils::interrupt_free,
    },
    chrono::{Datelike, NaiveDate, Timelike},
    core::{fmt::Write, str::FromStr},
    stm32h7xx_hal as hal,
};

pub const HELP: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
                        writeln!(m.writer(), "{help}")?;
                        return Ok(());
                    }
                }
                MenuItem::Alias { alias, command } => {
                    if *alias == program {
                        writeln!(m.writer(), "'{alias}' aliased to '{command}'")?;
                        m.run("help", &[*command])?;
                        return Ok(());
                    }
                }
            }
        }
        Err(MenuError::CommandNotFound)
    },
};

pub const PROGRAMS: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
                    writeln!(m.writer(), "{name:LABEL_WIDTH$} {description}")?;
                }
                MenuItem::Alias { alias, command } => {
                    writeln!(m.writer(), "{alias:LABEL_WIDTH$} aliased to {command}")?;
                }
            }
        }
        Ok(())
    },
};

pub const COMMANDS: MenuItem<'static, TerminalWriter> = MenuItem::Alias {
    alias: "commands",
    command: "programs",
};

pub const SYS: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
        ["loglevel"] => {
            writeln!(m.writer(), "Current log level: {}", logger::get_log_level())?;
            Ok(())
        }
        ["loglevel", level] => {
            match log::LevelFilter::from_str(level) {
                Ok(new_level) => {
                    logger::set_log_level(new_level);
                    writeln!(m.writer(), "New log level: {new_level}")?;
                }
                Err(e) => {
                    writeln!(m.writer(), "Failed to set new log level: {e}")?;
                }
            }
            Ok(())
        }
        _ => check_args_len(1, args.len()),
    },
};

pub const INFO: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "info",
    help: "info - Query information from the system",
    description: "Query information from the system",
    action: |m, args| match args {
        ["mcu"] => {
            writeln!(m.writer(), "{:LABEL_WIDTH$} STM32H747", "MCU")?;
            // Uid is 12 byts, hex string will be 24.
            let (id, id_len) = to_hex::<24>(hal::signature::Uid::read(), false);
            // SAFETY: to_hex always returns valid hex
            let id_str = unsafe { core::str::from_utf8_unchecked(&id[0..id_len]) };
            writeln!(m.writer(), "{:LABEL_WIDTH$} {}", "Unique ID", id_str)?;
            Ok(())
        }
        ["cpu"] => {
            writeln!(m.writer(), "{:LABEL_WIDTH$} Cortex-M7F", "Core")?;
            match interrupt_free(crate::system::cpu_freq) {
                Some(freq) => writeln!(
                    m.writer(),
                    "{:LABEL_WIDTH$} {}MHz",
                    "Core frequency",
                    freq.to_MHz()
                )?,
                None => writeln!(m.writer(), "{:LABEL_WIDTH$} unavailable", "Core frequency")?,
            }
            match interrupt_free(crate::system::cpu_temp) {
                Some(temp) => writeln!(
                    m.writer(),
                    "{:LABEL_WIDTH$} {:.01}°C",
                    "Core temperature",
                    temp
                )?,
                None => writeln!(
                    m.writer(),
                    "{:LABEL_WIDTH$} unavailable",
                    "Core temperature"
                )?,
            }
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Cycle count",
                cortex_m::peripheral::DWT::cycle_count()
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Instruction cache",
                bool_to_enabled_disabled_str(cortex_m::peripheral::SCB::icache_enabled())
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Data cache",
                bool_to_enabled_disabled_str(cortex_m::peripheral::SCB::dcache_enabled())
            )?;
            Ok(())
        }
        ["ram"] => {
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}KiB",
                "Internal RAM",
                crate::system::ram_size() / 1024
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}MiB",
                "External SDRAM",
                crate::mem::sdram::SDRAM_SIZE / (1024 * 1024)
            )?;
            Ok(())
        }
        ["flash"] => {
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}MiB",
                "Internal FLASH",
                crate::system::flash_size() / (1024 * 1024)
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}MiB",
                "External FLASH",
                crate::fs::qspi_store::QSPI_FLASH_SIZE / (1024 * 1024)
            )?;
            Ok(())
        }
        ["os"] => {
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {used} / {total} bytes ({fraction:.02}%)",
                "Memory usage",
                used = crate::mem::ALLOCATOR.used(),
                total = crate::mem::HEAP_SIZE,
                fraction =
                    (crate::mem::ALLOCATOR.used() as f64 / crate::mem::HEAP_SIZE as f64) * 100.0
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {} bytes",
                "GPU reserved",
                crate::display::FRAME_BUFFER_ALLOC_SIZE
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Rust version",
                consts::RUSTC_VERSION
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Version",
                consts::GIT_DESCRIBE
            )?;
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {}",
                "Debug",
                cfg!(debug_assertions)
            )?;
            let dt = NaiveDate::from_ymd_opt(
                consts::COMPILE_TIME_YEAR,
                consts::COMPILE_TIME_MONTH,
                consts::COMPILE_TIME_DAY,
            )
            .and_then(|dt| {
                dt.and_hms_opt(
                    consts::COMPILE_TIME_HOUR,
                    consts::COMPILE_TIME_MINUTE,
                    consts::COMPILE_TIME_SECOND,
                )
            })
            .unwrap();
            writeln!(
                m.writer(),
                "{:LABEL_WIDTH$} {weekday} {month} {day} {hh:02}:{mm:02}:{ss:02} {year}",
                "Compiled",
                weekday = dt.weekday(),
                month = month_to_str(dt.month()),
                day = dt.day(),
                hh = dt.hour(),
                mm = dt.minute(),
                ss = dt.second(),
                year = dt.year()
            )?;
            Ok(())
        }
        [] => {
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " MCU ")?;
            m.run("info", &["mcu"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " CPU ")?;
            m.run("info", &["cpu"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " RAM ")?;
            m.run("info", &["ram"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " FLASH ")?;
            m.run("info", &["flash"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " SD Card ")?;
            m.run("sdcard", &["info"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " OS ")?;
            m.run("info", &["os"])?;
            writeln!(m.writer(), "{:-^-HEADER_WIDTH$}", " Date/Time ")?;
            m.run("date", args)?;
            Ok(())
        }
        _ => {
            writeln!(m.writer(), "Unknown query")?;
            Ok(())
        }
    },
};
