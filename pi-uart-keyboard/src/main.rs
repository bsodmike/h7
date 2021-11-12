use std::io::Write;

use {
    rppal::uart::{Parity, Uart},
    std::io::Read,
};

fn main() {
    let mut uart = Uart::new(115_200, Parity::None, 8, 1).unwrap();

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
            Err(e) => eprintln!("Error: {:#?}", e),
        }

        match rx.try_recv() {
            Ok(s) => drop(uart.write(s.as_bytes()).unwrap()),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(e) => eprintln!("Error: {:#?}", e),
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // let mut stdin = std::io::stdin();

    // for bytes in stdin.lock().bytes() {
    //     match bytes {
    //         Ok(b) => println!("Data: {}", b),
    //         Err(e) => eprintln!("Error: {:#?}", e),
    //     }
    // }

    // loop {
    //     let mut buf = [0u8; 1];
    //     match stdin.read(&mut buf) {
    //         Ok(0) => {
    //             eprintln!("Got 0 bytes")
    //         }
    //         Ok(_) => {
    //             println!("Data: {}", buf[0])
    //         }
    //         Err(e) => {
    //             eprintln!("{:#?}", e)
    //         }
    //     }
    // }
}
