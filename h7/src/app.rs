use {crate::terminal::TerminalWriter, core::fmt::Write, h7_api::H7Api};

pub type AppEntry = extern "C" fn(*const h7_api::H7Api) -> u32;
pub const APP_START: *mut u8 = 0x2400_0000usize as *mut u8;
pub const APP_SIZE: usize = 512 * 1024;
// pub const APP_START: *mut u8 = 0x3000_0000usize as *mut u8;
// pub const APP_SIZE: usize = 288 * 1024;

pub static API: H7Api = H7Api { puts };

pub fn get_address(data: &[u8]) -> AppEntry {
    unsafe {
        // core::mem::transmute(
        //     ((data[3] as u32) << 24)
        //         | ((data[2] as u32) << 16)
        //         | ((data[1] as u32) << 8)
        //         | ((data[0] as u32) << 0),
        // )
        core::mem::transmute(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }
}

extern "C" fn puts(start: *const u8, len: usize) -> i32 {
    let s = unsafe { core::slice::from_raw_parts(start, len) };

    match core::str::from_utf8(s).map(|s| write!(TerminalWriter, "{}", s)) {
        Ok(Ok(_)) => 0,
        _ => -1,
    }
}
