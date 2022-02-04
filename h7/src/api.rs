use {crate::terminal::TerminalWriter, core::fmt::Write, h7_api::H7Api};

pub const API: *const H7Api = &H7Api { puts } as *const _;

extern "C" fn puts(start: *const u8, len: usize) -> i32 {
    let s = unsafe { core::slice::from_raw_parts(start, len) };

    match core::str::from_utf8(s).map(|s| write!(TerminalWriter, "{}", s)) {
        Ok(Ok(_)) => 0,
        _ => -1,
    }
}
