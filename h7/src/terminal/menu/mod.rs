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
    Group {
        title: &'i str,
        commands: &'i [MenuItem<'i, W>],
    },
}

pub struct Menu<'m, W: core::fmt::Write> {
    writer: W,
    menu: &'m [MenuItem<'m, W>],
}

impl<'m: 'i, 'i, W: core::fmt::Write> Menu<'m, W> {
    pub fn new(writer: W, menu: &'m [MenuItem<'i, W>]) -> Self {
        Self { writer, menu }
    }

    pub fn writer(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<'m, W: core::fmt::Write> Menu<'m, W> {
    pub fn run(&mut self, cmd: &str, args: &[&str]) -> MenuResult {
        fn run_impl<'m, W: core::fmt::Write>(
            menu: &mut Menu<'m, W>,
            cmd: &str,
            args: &[&str],
            menu_items: &[MenuItem<'m, W>],
        ) -> MenuResult {
            for item in menu_items {
                match item {
                    MenuItem::Command { name, action, .. } => {
                        if *name == cmd {
                            action(menu, args)?;
                            return Ok(());
                        }
                    }
                    MenuItem::Alias { alias, command } => {
                        if *alias == cmd {
                            menu.run(command, args)?;
                            return Ok(());
                        }
                    }
                    MenuItem::Group { commands, .. } => match run_impl(menu, cmd, args, commands) {
                        Ok(_) => return Ok(()),
                        Err(MenuError::CommandNotFound) => continue,
                        Err(e) => return Err(e),
                    },
                }
            }
            Err(MenuError::CommandNotFound)
        }

        run_impl(self, cmd, args, self.menu)
    }
}

impl<'m, W: core::fmt::Write> core::fmt::Write for Menu<'m, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.writer().write_str(s)
    }
}
