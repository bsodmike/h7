use {
    crate::led::Led,
    core::cell::RefCell,
    critical_section::{CriticalSection, Mutex},
    stm32h7xx_hal::crc::{Config, Crc},
};

pub static CRC: Mutex<RefCell<Option<Crc>>> = Mutex::new(RefCell::new(None));

pub fn crc(cs: CriticalSection, data: &[u8]) -> u32 {
    match *CRC.borrow(cs).borrow_mut() {
        Some(ref mut crc) => {
            let config = Config::new();
            crc.set_config(&config);
            crc.update_and_read(data)
        }
        None => 0,
    }
}

#[inline(always)]
pub fn interrupt_free<F, R>(f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    critical_section::with(f)
}

#[inline(always)]
pub fn into_ok_or_err<T>(result: Result<T, T>) -> T {
    match result {
        Ok(v) => v,
        Err(v) => v,
    }
}

pub fn write_pretty_duration<W: core::fmt::Write>(
    output: &mut W,
    dur: chrono::Duration,
) -> core::fmt::Result {
    // Note: The unwraps here are ok, an i64 can hold 292 years worth of nanoseconds.

    let days = dur.num_days();
    let hours = dur.num_hours() % 24;
    let minutes = dur.num_minutes() % 60;
    let seconds = dur.num_seconds() % 60;
    let milliseconds = dur.num_milliseconds() % 1000;
    let microseconds = dur.num_microseconds().unwrap() % 1000;
    let nanoseconds = dur.num_nanoseconds().unwrap() % 1000;

    if days > 0 {
        write!(output, "{days} days, {hours:02}:{minutes:02}:{seconds:02}")?;
    } else if hours > 0 {
        write!(output, "{hours:02}:{minutes:02}:{seconds:02}")?;
    } else if minutes > 0 {
        write!(output, "{minutes:02}:{seconds:02}")?;
    } else if seconds > 0 {
        write!(output, "{seconds}.{milliseconds}s")?;
    } else if milliseconds > 0 {
        write!(output, "{milliseconds}.{microseconds}ms")?;
    } else if microseconds > 0 {
        write!(output, "{microseconds}.{nanoseconds}us")?;
    } else {
        write!(output, "{nanoseconds}ns")?;
    }

    Ok(())
}
