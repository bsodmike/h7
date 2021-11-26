#[cfg(all(feature = "rpi", not(feature = "linux")))]
use rppal::uart::{Parity, Uart};

#[cfg(all(feature = "linux", not(feature = "rpi")))]
use std::io::ErrorKind;

use std::io::{Read, Write};

fn main() {
    #[cfg(all(feature = "rpi", not(feature = "linux")))]
    let mut uart = Uart::new(115_200, Parity::None, 8, 1).unwrap();
    #[cfg(all(feature = "linux", not(feature = "rpi")))]
    let mut uart = {
        let port = std::env::args().nth(1).expect("Expected serial device");
        serialport::new(port, 115_200).open_native().unwrap()
    };

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut s = String::new();
        match std::io::stdin().read_line(&mut s) {
            Ok(_) => tx.send(s).unwrap(),
            Err(e) => eprintln!("Error: {:#?}", e),
        }
    });

    loop {
        let mut buf = [0u8; 256];
        match uart.read(&mut buf) {
            Ok(0) => {}
            Ok(len) => {
                // println!("Data: {:?}", &buf[0..len]);
                match core::str::from_utf8(&buf[0..len]) {
                    Ok(s) => {
                        print!("{}", s);
                        std::io::stdout().flush().unwrap();
                    }
                    Err(e) => eprintln!("Error: {:#?}", e),
                }
            }
            #[cfg(all(feature = "linux", not(feature = "rpi")))]
            Err(e) if e.kind() == ErrorKind::TimedOut => {}
            #[cfg(all(feature = "linux", not(feature = "rpi")))]
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                eprintln!("Broken pipe");
                break;
            }
            Err(e) => eprintln!("Error: {:#?}", e),
        }

        match rx.try_recv() {
            Ok(s) => drop(uart.write(s.as_bytes()).unwrap()),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(e) => eprintln!("Error: {:#?}", e),
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
