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
    // Host::puts("Hello, World!\r\n");

    // loop {
    //     Host::delay(1);
    //     let kc = Host::getkc();
    //     if kc != 0 {
    //         Host::clear();
    //         if let Some(true) = core::char::from_u32(kc as u32).map(|c| c.is_ascii()) {
    //             if kc == 13 {
    //                 Host::putc(b'\r');
    //                 Host::putc(b'\n');
    //             } else {
    //                 Host::putc(kc as u8);
    //             }
    //         } else {
    //             let _ = write!(Host, "Got keycode {}\r\n", kc);
    //         }
    //     }
    // }

    5
}
