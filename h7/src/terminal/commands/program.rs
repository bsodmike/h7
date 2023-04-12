use {
    super::utils::*,
    crate::{
        app,
        fs::path::Path,
        led::Led,
        terminal::{
            menu::{MenuError, MenuItem},
            TerminalWriter, TERMINAL_INPUT_FIFO,
        },
        utils::interrupt_free,
    },
    core::fmt::Write,
};

pub const PLOAD: MenuItem<'static, TerminalWriter> = MenuItem::Command {
    name: "pload",
    help: "pload <dev:/path/to/bin.h7> - Load a program into ram",
    description: "Load a program into ram",
    action: |m, args| {
        check_args_len(1, args.len())?;
        let app_slice = app::app_slice();
        app_slice.fill(0); // .bss
        let path = Path::new(args[0]);
        match path.device() {
            Some("sdcard") => interrupt_free(|cs| {
                match crate::fs::sdmmc_fs::SD_CARD
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .map(|sdfs| sdfs.read_file(path, app_slice))
                {
                    Some(Ok(len)) => {
                        writeln!(m.writer(), "Program '{}' loaded ({} bytes)", args[0], len)?;
                        app::print_info(m.writer(), &app_slice[..len])?;
                        Ok(())
                    }
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

pub const PRUN: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
        writeln!(m.writer(), "Executing from {app_fn:p}")?;
        let ret = unsafe {
            Led::Green.on();
            Led::Red.on();
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
            // TODO: Clear input queue after app exit

            // Enable cache
            cp.SCB.enable_icache();
            cp.SCB.enable_dcache(&mut cp.CPUID);

            Led::Green.off();
            Led::Red.off();

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
            n => writeln!(m.writer(), "App leaked {n} bytes")?,
        }

        Ok(())
    },
};

pub const UPLOAD: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
        writeln!(m.writer(), "Read {n} bytes")?;
        if n > 8 {
            app::print_info(m.writer(), &app_slice[..n])?;
        } else {
            writeln!(m.writer(), "Not enough data")?;
        }

        Ok(())
    },
};
