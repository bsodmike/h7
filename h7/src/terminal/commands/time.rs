use {
    super::utils::*,
    crate::{
        terminal::{
            menu::{MenuError, MenuItem},
            TerminalWriter,
        },
        time::TimeSource,
    },
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    core::fmt::Write,
};

const DATE_PARSE_FORMAT: &str = "%Y-%m-%d";
const TIME_PARSE_FORMAT: &str = "%H:%M:%S";

pub const DATE: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
                } else if let Ok(time) = NaiveTime::parse_from_str(date_or_time, TIME_PARSE_FORMAT)
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
                    writeln!(m.writer(), "Invalid date or time '{date_or_time}'")
                }
            }
            ["set"] => {
                writeln!(m.writer(), "Set new system date and time:")?;
                writeln!(
                    m.writer(),
                    "Set date and time: date set {DATE_PARSE_FORMAT} {TIME_PARSE_FORMAT}"
                )?;
                writeln!(m.writer(), "Set date: date set {DATE_PARSE_FORMAT}")?;
                writeln!(m.writer(), "Set time: date set {TIME_PARSE_FORMAT}")
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
};

pub const CAL: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
                    } else if i == day + 1 && c % 7 != 1 {
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
};

pub const TIME: MenuItem<'static, TerminalWriter> = MenuItem::Command {
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
};
