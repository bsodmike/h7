use crate::Host;

pub const MALLOC_DEFAULT_ALIGN: usize = 8;

// Sys, Mem
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn h7_malloc(size: usize) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align_unchecked(size, MALLOC_DEFAULT_ALIGN);
    Host::alloc(layout)
}

#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn h7_malloc_aligned(size: usize, align: usize) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align_unchecked(size, align);
    Host::alloc(layout)
}

#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn h7_free(ptr: *mut u8) {
    Host::free(ptr)
}

#[no_mangle]
pub unsafe extern "C" fn h7_panic(msg: *const u8) -> ! {
    let slice = core::slice::from_raw_parts(msg, cstd::strlen(msg));
    let str_slice = core::str::from_utf8_unchecked(slice);
    Host::panic(str_slice)
}

// IO
#[no_mangle]
pub unsafe extern "C" fn h7_getc() -> u8 {
    Host::getc()
}

#[no_mangle]
pub unsafe extern "C" fn h7_putc(c: u8) -> i32 {
    Host::putc(c)
}

#[no_mangle]
pub unsafe extern "C" fn h7_puts(msg: *const u8) -> i32 {
    let slice = core::slice::from_raw_parts(msg, cstd::strlen(msg));
    let str_slice = core::str::from_utf8_unchecked(slice);
    Host::puts(str_slice)
}

mod cstd {
    pub(crate) unsafe fn strlen(s: *const u8) -> usize {
        let mut result = 0;
        while *s.add(result) != 0 {
            result += 1;
        }
        result
    }
}
