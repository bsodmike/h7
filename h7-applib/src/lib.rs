#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(
    all(feature = "alloc", feature = "default-alloc-handler"),
    feature(alloc_error_handler)
)]

#[cfg(feature = "c-api")]
pub mod c_api;

#[cfg(feature = "alloc")]
extern crate alloc;

pub struct Host;

#[cfg(all(feature = "alloc"))]
mod h7_alloc {
    struct H7Allocator;

    #[global_allocator]
    static A: H7Allocator = H7Allocator;

    unsafe impl core::alloc::GlobalAlloc for H7Allocator {
        #[inline(always)]
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            super::Host::alloc(layout)
        }

        #[inline(always)]
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
            super::Host::free(ptr)
        }
    }
}

use {
    core::mem::MaybeUninit,
    h7_api::{AppEntryPoint, H7Api},
};

#[link_section = ".entry_point"]
#[no_mangle]
#[used]
pub static ENTRY_POINT: AppEntryPoint = entry_point;
static mut API_POINTER: MaybeUninit<&'static H7Api> = MaybeUninit::uninit();

/// The function called by the host to start us up. Does some setup, then
/// jumps to a function called `h7_main` defined by the actual application using
/// this crate.
#[no_mangle]
extern "C" fn entry_point(table: *const H7Api) -> i32 {
    // Turn the pointer into a reference and store in a static.
    unsafe {
        API_POINTER.write(&*table);
    };

    extern "C" {
        fn h7_main() -> i32;
    }
    // Call the user application
    unsafe { h7_main() }
}

#[inline(always)]
fn get_api() -> &'static H7Api {
    unsafe { API_POINTER.assume_init() }
}

impl Host {
    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub(crate) unsafe fn alloc(layout: core::alloc::Layout) -> *mut u8 {
        (get_api().alloc)(layout.size(), layout.align())
    }

    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub(crate) unsafe fn free(ptr: *mut u8) {
        (get_api().free)(ptr)
    }

    #[inline(always)]
    pub fn panic(msg: &str) -> ! {
        (get_api().panic)(msg.as_ptr(), msg.len())
    }

    #[inline(always)]
    pub fn getc() -> u8 {
        (get_api().getc)()
    }

    #[inline(always)]
    pub fn putc(c: u8) -> i32 {
        (get_api().putc)(c)
    }

    #[inline(always)]
    pub fn puts(s: &str) -> i32 {
        (get_api().puts)(s.as_ptr(), s.len())
    }
}

impl core::fmt::Write for Host {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::puts(s);
        Ok(())
    }
}

#[cfg(all(
    feature = "alloc",
    feature = "default-alloc-handler",
    target_os = "none"
))]
#[inline(never)]
#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    Host::panic("Allocation failed")
}

#[cfg(all(feature = "default-panic-handler", target_os = "none"))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    Host::panic("User application paniced")
}
