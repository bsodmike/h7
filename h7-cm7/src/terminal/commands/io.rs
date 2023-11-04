use {
    super::utils::from_hex,
    crate::{
        fs::{
            path::Path,
            qspi_store::{mx25l::status as mx25l_status, QSPI_STORE},
            sdmmc_fs,
        },
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
    help: "ls <device>:<dir> - List files on a filesystem",
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
            Some("nor") => {
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

pub const MV: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "mv",
    help: "mv <source> <destination> - Move a file or directory from source to destination",
    description: "(TODO) Move a file or directory",
    action: |m, _| {
        writeln!(m.writer(), "todo")?;
        Ok(())
    },
};

pub const RM: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "rm",
    help: "rm <file> - Remove a file from a filesystem",
    description: "(TODO) Remove a file from a filesystem",
    action: |m, _| {
        writeln!(m.writer(), "todo")?;
        Ok(())
    },
};

pub const CP: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "cp",
    help: "cp <source> <destination> - Copy a file or directory from source to destination",
    description: "(TODO) Copy a file or directory",
    action: |m, _| {
        writeln!(m.writer(), "todo")?;
        Ok(())
    },
};

pub const CAT: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "cat",
    help: "cat - Read and print a file to stdout",
    description: "(TODO) Read and print a file to stdout",
    action: |m, _| {
        writeln!(m.writer(), "todo")?;
        Ok(())
    },
};

pub const NOR: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "nor",
    help: "nor <(i|info)|(m|mount)|(u|unmount)|(f|format)> - Info/Mount/Unmount/Format NOR-Flash filesystem",
    description: "Info/Mount/Unmount/Format NOR-Flash filesystem",
    action: |m, args| {
        // writeln!(m.writer(), "todo")?;

        match args {
            ["dev", "ce"] => {
                let result = interrupt_free(|cs| {
                    QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().chip_erase()
                });
                writeln!(m.writer(), "{result:?}")?;
            }
            ["dev", "reset"] => {
                let result = interrupt_free(|cs| {
                    QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().reset()
                });
                writeln!(m.writer(), "{result:?}")?;
            }
            ["dev", "id"] => {
                let result = interrupt_free(|cs| {
                    QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().reaq_identification()
                });
                match result {
                    Ok([mfn_id, mem_type, mem_density]) => {
                        writeln!(m.writer(), "Manufacturer ID: {mfn_id:02x}, Memory type: {mem_type:02x}, Memory density: {mem_density:02x}")?;
                    }
                    Err(e) => {
                        writeln!(m.writer(), "Error: {e:?}")?;
                    }
                }
            },
            ["dev", "config"] => {
                let result = interrupt_free(|cs| {
                    QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().read_config()
                });
                match result {
                    Ok(config) => {
                        writeln!(m.writer(), "Config: {config:08b}")?;
                        writeln!(
                            m.writer(),
                            "DC={dc:02b} TB={tb}, ODS={ods:03b}",
                            dc = (config & 0b1100_0000) >> 6,
                            tb = if (config & 0b0000_1000) > 0 { "1" } else { "0" },
                            ods = (config & 0b0000_0111)
                        )?;
                    }
                    Err(e) => {
                        writeln!(m.writer(), "Error: {e:?}")?;
                    }
                }
            },
            ["dev", "status"] => {
                let result = interrupt_free(|cs| {
                    QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().read_status()
                });
                match result {
                    Ok(status) => {
                        writeln!(m.writer(), "Status: {status:08b}")?;
                        writeln!(
                            m.writer(),
                            "SRWD={srwd} QE={qe} BP3={bp3} BP2={bp2} BP1={bp1} BP0={bp0} WEL={wel} WIP={wip}",
                            srwd = if (status & mx25l_status::SRWD) > 0 { "1" } else { "0" },
                            qe = if (status & mx25l_status::QE) > 0 { "1" } else { "0" },
                            bp3 = if (status & mx25l_status::BP3) > 0 { "1" } else { "0" },
                            bp2 = if (status & mx25l_status::BP2) > 0 { "1" } else { "0" },
                            bp1 = if (status & mx25l_status::BP1) > 0 { "1" } else { "0" },
                            bp0 = if (status & mx25l_status::BP0) > 0 { "1" } else { "0" },
                            wel = if (status & mx25l_status::WEL) > 0 { "1" } else { "0" },
                            wip = if (status & mx25l_status::WIP) > 0 { "1" } else { "0" }
                        )?;
                    }
                    Err(e) => {
                        writeln!(m.writer(), "Error: {e:?}")?;
                    }
                }
            },
            ["dev", "read", address_str, length_str] => {
                let address = u32::from_str_radix(address_str, 16).map_err(|_| MenuError::InvalidArgument)?;
                let length = length_str.parse::<u32>().map_err(|_| MenuError::InvalidArgument)?;
                const OUTPUT_WIDTH: u32 = 16;
                let mut ascii_rep = [b'.'; OUTPUT_WIDTH as usize];
                for i in 0..length {
                    let mut data = [0u8; 1];
                    let result = interrupt_free(|cs| {
                        QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().read(address + i, &mut data)
                    });
                    if (address + i) % OUTPUT_WIDTH == 0 && i != 0 {
                        writeln!(m.writer())?;
                    }

                    if (address + i) % OUTPUT_WIDTH == 0 || i == 0 {
                        write!(m.writer(), "{:06x}  ", address + i)?;
                        ascii_rep.fill(b'.');
                    }

                    if address % OUTPUT_WIDTH != 0 && i == 0 {
                        for _ in 0..(address % OUTPUT_WIDTH) {
                            write!(m.writer(), "   ")?;
                        }
                    }

                    match result {
                        Ok(_) => {
                            write!(m.writer(), "{:02x} ", data[0])?;
                            if data[0].is_ascii_graphic() {
                                ascii_rep[((address + i) % OUTPUT_WIDTH) as usize] = data[0];
                            }
                            if (address + i + 1) % OUTPUT_WIDTH == 0 {
                                // write!(m.writer(), " |LAST|")?;
                                if let Ok(s) = core::str::from_utf8(&ascii_rep) {
                                    write!(m.writer(), " |{s}|")?;
                                }
                            }
                        },
                        Err(_e) => {
                            // writeln!(m.writer(), "Error: {e:?}")?;
                            write!(m.writer(), "-- ")?;
                            break;
                        }
                    }
                }
                if (address + length) % OUTPUT_WIDTH != 0 {
                    for _ in 0..(OUTPUT_WIDTH - ((address + length) % OUTPUT_WIDTH)) {
                        write!(m.writer(), "   ")?;
                    }
                    if let Ok(s) = core::str::from_utf8(&ascii_rep) {
                        write!(m.writer(), " |{s}|")?;
                    }
                }
                writeln!(m.writer())?;
            },
            ["dev", "write", address_str, hex_str] => {
                let address = u32::from_str_radix(address_str, 16).map_err(|_| MenuError::InvalidArgument)?;
                let all_bytes_hex = hex_str.chars().all(|c| c.is_ascii_hexdigit());
                if !all_bytes_hex || hex_str.len() % 2 != 0 {
                    return Err(MenuError::InvalidArgument);
                }

                for (offset, [upper, lower]) in hex_str.as_bytes().array_chunks::<2>().enumerate() {
                    let byte = from_hex(*upper, *lower).unwrap();
                    write!(m.writer(), "0x{byte:02x} ")?;
                    let result = interrupt_free(|cs| {
                        QSPI_STORE.borrow(cs).borrow_mut().as_deref_mut().unwrap().write(address + offset as u32, &[byte])
                    });

                    if let Err(e) = result {
                        writeln!(m.writer(), "Error: {e:?}")?;
                        break;
                    }
                }
                writeln!(m.writer())?;
            },
            _ => {
                return Err(MenuError::InvalidArgument)
            }
        }

        Ok(())
    },
};

pub const SDCARD: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "sdcard",
    help: "sdcard <(i|info)|(m|mount)|(u|unmount)> - Info/Mount/Unmount SD Card",
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

pub const CURL: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "curl",
    help: "curl - (TODO) Fetch content from a remote machine over HTTP/HTTPS",
    description: "(TODO) Fetch content from a remote machine over HTTP/HTTPS",
    action: |m, _| {
        writeln!(m.writer(), "todo")?;
        Ok(())
    },
};
