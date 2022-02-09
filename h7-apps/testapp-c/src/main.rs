#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// Make rustc happy
use h7_applib as _;

// If you want to dynamically allocate memory, make sure to configure the allocator
// #[global_allocator]
// static A: H7Allocator = H7Allocator;

#[cfg(not(target_os = "none"))]
pub fn main() {
    h7_applib::Host::init();
    extern "C" {
        fn h7_main() -> i32;
    }
    let r = unsafe { h7_main() };
    std::process::exit(r);
}

// The entry point is in the main.c file
// #[no_mangle]
// pub extern "C" fn h7_main() -> i32 {}
