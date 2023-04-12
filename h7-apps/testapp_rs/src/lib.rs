#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use {core::fmt::Write, h7_applib::Host};

extern crate alloc;

#[inline(never)]
fn mul(a: usize, b: usize) -> usize {
    a * b
}

#[no_mangle]
pub extern "C" fn h7_main() -> i32 {
    Host::puts("Hello from Rust test app!\n");

    let stack_var = 5;

    let _ = writeln!(Host, "mul: {:p}", &mul);
    let _ = writeln!(Host, "h7_main: {:p}", &h7_main);
    let _ = writeln!(Host, "stack_var: {:p}", &stack_var);

    // let s = alloc::string::String::from("Allocated string\n");
    // Host::puts(&s);

    // Host::putc(b'\n');

    // let v = alloc::vec::Vec::<u8>::with_capacity(128).leak();
    // let _ = writeln!(Host, "vptr: {:p}", v);

    // loop {
    //     let c = Host::getc();

    //     if c == b'b' {
    //         break;
    //     } else if c == b'\r' || c == b'\n' {
    //         Host::putc('\n');
    //     } else if c != 0 {
    //         Host::putc(c);
    //     };
    // }

    0
}
