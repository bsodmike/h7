#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use {core::fmt::Write, h7_applib::Host};

extern crate alloc;

#[cfg(not(target_os = "none"))]
pub fn main() {
    h7_applib::sim::main()
}

#[no_mangle]
pub extern "C" fn h7_main() -> i32 {
    Host::puts("Hello from Rust test app!\n");

    let s = alloc::string::String::from("Allocated string\n");
    Host::puts(&s);

    Host::putc(b'\n');

    // let v = alloc::vec::Vec::<u8>::with_capacity(128).leak();
    // let _ = writeln!(Host, "vptr: {:p}", v);

    loop {
        let c = Host::getc();

        if c == b'b' {
            break;
        } else if c != 0 {
            Host::putc(c);
        }

        // Host::delay(10);
    }

    0
}
