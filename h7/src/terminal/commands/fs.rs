use {
    crate::{
        fs::{path::Path, sdmmc_fs},
        terminal::{
            commands::LABEL_WIDTH,
            menu::{MenuError, MenuItem},
            TerminalWriter,
        },
        utils::interrupt_free,
    },
    core::fmt::Write,
    fugit::RateExtU32,
    stm32h7xx_hal as hal,
};

pub const LS: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "ls",
    help: "List files",
    description: "List files",
    action: |m, args| {
        let path = Path::new(args.first().unwrap_or(&""));
        match path.device() {
            Some("sdcard") => interrupt_free(|cs| {
                match crate::fs::sdmmc_fs::SD_CARD
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .map(|sdfs| {
                        sdfs.ls(path, |e| {
                            let _ = sdmmc_fs::print_dir_entry(m.writer(), e);
                        })
                    }) {
                    Some(Ok(_)) => Ok(()),
                    Some(Err(e)) => {
                        writeln!(m.writer(), "Error: {e}")?;
                        Ok(())
                    }
                    None => {
                        writeln!(m.writer(), "Error: SD Card controller not initialized")?;
                        Ok(())
                    }
                }
            }),
            Some("nor") | Some("flash") => {
                writeln!(m.writer(), "Not implemented")?;
                Ok(())
            }
            Some(device) => {
                writeln!(m.writer(), "Unknown device '{device}'")?;
                Ok(())
            }
            None => {
                writeln!(m.writer(), "No device selected")?;
                Ok(())
            }
        }
    },
};

pub const SDCARD: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "sdcard",
    help: "sdcard <action> - Mount/Unmount SD Card",
    description: "Mount/Unmount SD Card",
    action: |m, args| match args {
        ["i" | "info"] => match interrupt_free(|cs| {
            crate::fs::sdmmc_fs::SD_CARD
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .map(|sdfs| (sdfs.is_mounted(), sdfs.card_size()))
        }) {
            Some((mounted, size)) => {
                writeln!(m.writer(), "{:LABEL_WIDTH$} {}", "SD Card mounted", mounted)?;
                match size {
                    Ok(bytes) => {
                        writeln!(
                            m.writer(),
                            "{:LABEL_WIDTH$} {}GiB",
                            "Size",
                            bytes as f64 / (1024 * 1024 * 1024) as f64
                        )?;
                        Ok(())
                    }
                    Err(_) => {
                        writeln!(m.writer(), "{:LABEL_WIDTH$} <unavailable>", "Size")?;
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
            writeln!(m.writer(), "Attempting to mount SD Card @ {freq}kHz")?;
            match crate::fs::sdmmc_fs::SD_CARD
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .map(|sdfs| sdfs.mount::<hal::delay::Delay, _>(freq.kHz(), 10, None))
            {
                Some(Ok(_)) => {
                    writeln!(m.writer(), "SD Card mounted")?;
                    Ok(())
                }
                Some(Err(e)) => {
                    writeln!(m.writer(), "{e}")?;
                    Ok(())
                }
                None => {
                    writeln!(m.writer(), "SD Card controller not initialized")?;
                    Err(MenuError::CommandError(None))
                }
            }
        }),
        ["u" | "unmount"] => interrupt_free(|cs| {
            match crate::fs::sdmmc_fs::SD_CARD
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
                    writeln!(m.writer(), "{e}")?;
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
};
