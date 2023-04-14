use {
    crate::terminal::menu::{Menu, MenuError, MenuItem, MenuResult},
    chrono::{Datelike, NaiveDate},
};

pub struct PaddedStr<'s, const PADDING: u8>(pub &'s str, pub usize);

impl<'s, const PADDING: u8> core::fmt::Display for PaddedStr<'s, PADDING> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // If only there was a trait to hint the size of the formatted output 🙃
        const BUFFER_LEN: usize = 64;
        debug_assert!(BUFFER_LEN >= self.0.len() + self.1 + self.1);
        let mut buf = [PADDING; BUFFER_LEN];
        buf[self.1..(self.1 + self.0.len())].copy_from_slice(self.0.as_bytes());
        if let Ok(s) = core::str::from_utf8(&buf[..self.0.len() + self.1 + self.1]) {
            f.pad(s)?;
        }
        Ok(())
    }
}

pub fn iter_menu<'m, W: core::fmt::Write, F>(
    menu: &mut Menu<'m, W>,
    args: &[&str],
    menu_items: &[MenuItem<'m, W>],
    cb: &mut F,
) -> MenuResult
where
    F: FnMut(
        &mut Menu<'m, W>,
        &[&str],
        &MenuItem<'m, W>,
        &[MenuItem<'m, W>],
        usize,
    ) -> Result<bool, MenuError>,
{
    fn iter_menu_impl<'m, W: core::fmt::Write, F>(
        menu: &mut Menu<'m, W>,
        args: &[&str],
        menu_items: &[MenuItem<'m, W>],
        cb: &mut F,
        level: usize,
    ) -> MenuResult
    where
        F: FnMut(
            &mut Menu<'m, W>,
            &[&str],
            &MenuItem<'m, W>,
            &[MenuItem<'m, W>],
            usize,
        ) -> Result<bool, MenuError>,
    {
        for item in menu_items {
            match item {
                MenuItem::Group { commands, .. } => {
                    if cb(menu, args, item, menu_items, level)? {
                        iter_menu_impl(menu, args, commands, cb, level + 1)?;
                    }
                }
                _ => {
                    if cb(menu, args, item, menu_items, level)? {
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    iter_menu_impl(menu, args, menu_items, cb, 0)
}

pub fn month_to_str(month: u32) -> &'static str {
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

pub fn days_in_month<D: Datelike>(d: &D) -> u32 {
    let y = d.year();
    let m = d.month();
    if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap()
    }
    .signed_duration_since(NaiveDate::from_ymd_opt(y, m, 1).unwrap())
    .num_days() as u32
}

pub fn bool_to_enabled_disabled_str(b: bool) -> &'static str {
    match b {
        true => "enabled",
        false => "disabled",
    }
}

pub const fn nibble_to_char(nibble: u8, lowercase: bool) -> Option<u8> {
    match nibble & 0x0F {
        0..=9 => Some(nibble + 48),
        10..=15 => Some(nibble + 55 + if lowercase { 32 } else { 0 }),
        _ => None,
    }
}

pub fn to_hex<const N: usize>(data: &[u8], lowercase: bool) -> ([u8; N], usize) {
    let mut res = [0u8; N];
    let len = data.len().min(N / 2);
    for (i, byte) in data.iter().enumerate() {
        let idx = i * 2;
        // SAFETY: These unwraps can never fail as long as nibble_to_char handles the range 0..=15
        unsafe {
            res[idx] = nibble_to_char(byte >> 4, lowercase).unwrap_unchecked();
            res[idx + 1] = nibble_to_char(byte & 0x0f, lowercase).unwrap_unchecked();
        }
    }
    (res, len * 2)
}

pub const fn from_hex(nibble1: u8, nibble2: u8) -> Option<u8> {
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

pub const fn check_args_len(expected: u8, actual: usize) -> MenuResult {
    if (actual as u8) > expected {
        Err(MenuError::TooManyArgs(expected, actual as u8))
    } else if (actual as u8) < expected {
        Err(MenuError::NotEnoughArgs(expected, actual as u8))
    } else {
        Ok(())
    }
}
