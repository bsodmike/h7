use {
    crate::{terminal::TerminalWriter, utils},
    core::fmt::Write,
    h7_api::{AppEntryPoint, H7Api},
};

pub const APP_ADDR_ALIGN: usize = 4;

pub const APP_START: *mut u8 = 0x2400_0000usize as *mut u8;
pub const APP_SIZE: usize = 512 * 1024;

// pub const APP_START: *mut u8 = 0x3000_0000usize as *mut u8;
// pub const APP_SIZE: usize = 128 * 1024;

pub static API: H7Api = H7Api { puts };

pub fn get_address(data: &[u8]) -> AppEntryPoint {
    unsafe {
        let addr = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let ptr = addr as *const ();
        core::mem::transmute(ptr)
    }
}

pub fn check_address(addr: AppEntryPoint) -> bool {
    unsafe {
        let ptr = addr as *const u8;
        // LSB is not part of the actual address,
        // but rather indicate if the cpu should
        // switch to arm or thumb mode.
        // 0 = ARM, 1 = THUMB
        let aligned = ((ptr as usize & 0xffff_fffe) % APP_ADDR_ALIGN) == 0;
        let addr_after_start = ptr.add(4).ge(&(APP_START as *const _));
        let addr_before_end = ptr < (APP_START.add(APP_SIZE));
        aligned && addr_after_start && addr_before_end
    }
}

pub fn app_slice() -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(APP_START, APP_SIZE) }
}

pub fn verify_app(slice: &[u8]) -> Result<u32, u32> {
    let len = slice.len();
    let calculated_crc = utils::interrupt_free(|cs| utils::crc(cs, &slice[..(len - 4)]));
    let provided_crc = u32::from_be_bytes([
        slice[len - 4],
        slice[len - 3],
        slice[len - 2],
        slice[len - 1],
    ]);
    if calculated_crc == provided_crc {
        Ok(calculated_crc)
    } else {
        Err(calculated_crc)
    }
}

pub fn print_info<W: core::fmt::Write>(w: &mut W, data: &[u8]) -> core::fmt::Result {
    let crc = verify_app(data);
    let addr = get_address(data);
    writeln!(
        w,
        "Address: {addr:p} ({addr_check}), CRC: 0x{crc:08x} ({crc_check}), Size: 0x{size:x}",
        addr_check = if check_address(addr) {
            if (addr as usize) % 2 == 1 {
                "valid thumb"
            } else {
                "valid arm"
            }
        } else {
            "invalid"
        },
        crc = crc.into_ok_or_err(),
        crc_check = if crc.is_ok() { "passed" } else { "failed" },
        size = data.len()
    )
}

extern "C" fn puts(start: *const u8, len: usize) -> i32 {
    let s = unsafe { core::slice::from_raw_parts(start, len) };

    match core::str::from_utf8(s).map(|s| write!(TerminalWriter, "{}", s)) {
        Ok(Ok(_)) => 0,
        _ => -1,
    }
}
