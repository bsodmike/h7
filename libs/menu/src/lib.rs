#![no_std]

mod error;
pub use error::{MenuError, MenuResult};

pub type MenuAction<W> = fn(writer: &mut Menu<W>, args: &[&str]) -> MenuResult;

pub enum MenuItem<'i, W: core::fmt::Write> {
    Command {
        name: &'i str,
        help: &'i str,
        description: &'i str,
        action: MenuAction<W>,
    },
    Alias {
        alias: &'i str,
        command: &'i str,
    },
}

pub struct Menu<'m, W: core::fmt::Write> {
    writer: W,
    menu: &'m [MenuItem<'m, W>],
}

impl<'m, W: core::fmt::Write> Menu<'m, W> {
    pub fn new(writer: W, menu: &'m [MenuItem<'m, W>]) -> Self {
        Self { writer, menu }
    }

    pub fn writer(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<'m, W: core::fmt::Write> Menu<'m, W> {
    pub fn run(&mut self, cmd: &str, args: &[&str]) -> MenuResult {
        for item in self.menu {
            match item {
                MenuItem::Command { name, action, .. } => {
                    if *name == cmd {
                        action(self, args)?;
                        return Ok(());
                    }
                }
                MenuItem::Alias { alias, command } => {
                    if *alias == cmd {
                        self.run(*command, args)?;
                        return Ok(());
                    }
                }
            }
        }
        Err(MenuError::CommandNotFound)
    }
}

impl<'m, W: core::fmt::Write> core::fmt::Write for Menu<'m, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.writer().write_str(s)
    }
}

pub const fn check_args_len(expected: u8, actual: usize) -> MenuResult {
    if (actual as u8) > expected {
        Err(MenuError::TooManyArgs(expected, actual as u8))
    } else if (actual as u8) < expected {
        Err(MenuError::NotEnoughArgs(expected, actual as u8))
    } else {
        Ok(())
    }
}
