# Menu system for H7

```rust
use {
    core::fmt::Write,
    menu::{check_args_len, Menu, MenuError, MenuItem},
    std::io::Write as IoWrite,
};

mod menu;

pub struct ConsoleWriter;

impl core::fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

const MENU: &[MenuItem<ConsoleWriter>] = &[
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
                            m.run("help", &[program])?;
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
            const PROG_NAME_MAX: usize = 30;
            for item in MENU {
                match item {
                    MenuItem::Command {
                        name, description, ..
                    } => {
                        write!(m.writer(), "{}", name)?;
                        for _ in 0..(PROG_NAME_MAX - name.len()) {
                            write!(m.writer(), " ")?;
                        }
                        writeln!(m.writer(), "{}", description)?;
                    }
                    MenuItem::Alias { alias, command } => {
                        write!(m.writer(), "{}", alias)?;
                        for _ in 0..(PROG_NAME_MAX - alias.len()) {
                            write!(m.writer(), " ")?;
                        }
                        writeln!(m.writer(), "aliased to {}", command)?;
                    }
                }
            }
            Ok(())
        },
    },
    MenuItem::Command {
        name: "exit",
        help: "exit - Exit the shell",
        description: "Exit the shell",
        action: |m, args| {
            check_args_len(0, args.len())?;
            writeln!(m.writer(), "Bye")?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "pload",
        help: "pload <path/to/bin.h7> - Load a program into ram",
        description: "Load a program into ram",
        action: |_, args| {
            check_args_len(1, args.len())?;
            Ok(())
        },
    },
    MenuItem::Command {
        name: "prun",
        help: "prun - Run program loaded in ram",
        description: "Run program loaded in ram",
        action: |_, _| Ok(()),
    },
    MenuItem::Alias {
        alias: "commands",
        command: "programs",
    },
];

fn main() {
    let mut menu = Menu::new(ConsoleWriter, MENU);

    loop {
        let mut cmd = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut cmd).unwrap();
        let parts = cmd
            .trim()
            .split_whitespace()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>();

        // println!("{:?}", parts);

        if let Some(cmd) = parts.get(0) {
            if let Err(e) = menu.run(*cmd, &parts[1..]) {
                eprintln!("{}", e);
            }
            if *cmd == "exit" {
                break;
            }
        }
    }
}

```
