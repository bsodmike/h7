#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use {core::fmt::Write, h7_app::Host};

#[cfg(not(target_os = "none"))]
pub fn main() {
    Host::init();
    let r = h7_main();
    std::process::exit(r);
}

#[no_mangle]
pub extern "C" fn h7_main() -> i32 {
    Host::puts("Hello, World!\r\n");

    loop {
        Host::delay(1);
        let kc = Host::getkc();
        if kc != 0 {
            Host::clear();
            let _ = write!(Host, "Got keycode {}\r\n", kc);
        }
    }

    0
}
