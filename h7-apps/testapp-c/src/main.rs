#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

// The entry point is in the main.c file

// Make rustc happy
use h7_applib as _;

#[cfg(not(target_os = "none"))]
pub fn main() {
    h7_applib::sim::main()
}
