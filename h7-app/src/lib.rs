#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
mod display;

pub struct Host;

/// Implementation used when building code for the H7
#[cfg(target_os = "none")]
pub mod target {

    use super::*;
    use h7_api::H7Api;

    #[link_section = ".entry_point"]
    #[no_mangle]
    #[used]
    /// The pointer Monotron calls to start running this application.
    pub static ENTRY_POINT: extern "C" fn(*const H7Api) -> i32 = entry_point;
    static mut API_POINTER: Option<&'static H7Api> = None;

    #[no_mangle]
    /// The function called by the host to start us up. Does some setup, then
    /// jumps to a function called `main` defined by the actual application using
    /// this crate.
    pub extern "C" fn entry_point(table: *const H7Api) -> i32 {
        // Turn the pointer into a reference and store in a static.
        unsafe {
            API_POINTER = Some(&*table);
        };

        extern "C" {
            fn h7_main() -> i32;
        }
        // call the user application
        unsafe { h7_main() }
    }

    fn get_api() -> &'static H7Api {
        unsafe {
            if let Some(api) = API_POINTER {
                api
            } else {
                unreachable!()
            }
        }
    }

    impl Host {
        pub fn delay(ms: u32) {}

        pub fn puts(s: &str) {
            (get_api().puts)(s.as_ptr(), s.len());
        }

        pub fn clear() -> i32 {
            0
        }

        pub fn getkc() -> i32 {
            0
        }
    }

    impl core::fmt::Write for Host {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            Self::puts(s);
            Ok(())
        }
    }

    #[inline(never)]
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        loop {
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        }
    }
}

/// Implementation used when building code for Linux/Windows
#[cfg(not(target_os = "none"))]
pub mod target {

    use {
        super::{display::*, *},
        embedded_graphics::{
            mono_font::iso_8859_10::FONT_10X20 as ISO_FONT_10X20,
            pixelcolor::{raw::RawU16, Rgb565},
            prelude::*,
        },
        embedded_graphics_simulator::{
            OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
        },
        std::{
            sync::{
                atomic::{AtomicBool, Ordering},
                mpsc::{channel, Receiver},
                Arc, Mutex,
            },
            time::Duration,
        },
    };

    struct Context {
        x: u32,
        y: u32,
    }

    const GLYPH_WIDTH: usize = 10;
    const GLYPH_HEIGHT: usize = 20;
    const WIDTH: u32 = 1280;
    const HEIGHT: u32 = 768;
    const ROWS: usize = HEIGHT as usize / GLYPH_HEIGHT;
    const COLS: usize = WIDTH as usize / GLYPH_WIDTH;

    static mut TEXT_GRID: [[u8; COLS]; ROWS] = [[b' '; COLS]; ROWS];
    static mut KC_RECEIVER: Option<Receiver<i32>> = None;
    static mut CONTEXT: Context = Context { x: 0, y: 0 };
    static mut DISPLAY: Option<Arc<Mutex<Display<SimulatorDisplay<Rgb565>>>>> = None;
    static REDRAW: AtomicBool = AtomicBool::new(true);

    fn draw_text_grid() {
        let mut lock = unsafe { DISPLAY.as_mut().unwrap().lock().unwrap() };

        for (n, line) in unsafe { TEXT_GRID }.iter().enumerate() {
            if let Ok(s) = core::str::from_utf8(line) {
                let _ = lock.draw_text(
                    s,
                    &ISO_FONT_10X20,
                    Rgb565::from(RawU16::new(0xffff)),
                    XPos::Absolute(0),
                    YPos::Absolute((n * GLYPH_HEIGHT) as i32 + 18),
                );
            }
        }
    }

    impl Host {
        pub fn init() {
            let (tx, rx) = channel();
            unsafe {
                KC_RECEIVER = Some(rx);
                DISPLAY = Some(Arc::new(Mutex::new(Display::new(
                    SimulatorDisplay::<Rgb565>::new(Size::new(WIDTH, HEIGHT)),
                ))))
            };

            let disp = Arc::clone(unsafe { DISPLAY.as_ref().unwrap() });
            std::thread::spawn(move || {
                let output_settings = OutputSettingsBuilder::new().build();
                let mut w = Window::new(env!("CARGO_PKG_NAME"), &output_settings);

                loop {
                    if let Ok(_) =
                        REDRAW.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    {
                        println!("REDRAW!");
                        draw_text_grid();
                        w.update(&disp.lock().unwrap().display);
                    }
                    for e in w.events() {
                        match e {
                            SimulatorEvent::Quit => std::process::exit(1),
                            SimulatorEvent::KeyDown { keycode, .. } => {
                                // HOME key
                                if keycode as i32 == 1073741898 {
                                    REDRAW.store(true, Ordering::SeqCst);
                                }
                                drop(tx.send(keycode as i32))
                            }
                            _ => {}
                        }
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            });
        }

        pub fn delay(ms: u32) {
            std::thread::sleep(std::time::Duration::from_millis(ms as u64))
        }

        pub fn getc() -> u8 {
            0
        }

        pub fn putc(c: u8) -> i32 {
            // print!("{}", c as char);
            match c {
                b'\r' => unsafe { CONTEXT.x = 0 },
                b'\n' => unsafe {
                    CONTEXT.y = (CONTEXT.y + 1) % ROWS as u32;
                },
                b'\t' => unsafe { CONTEXT.x = (CONTEXT.x + 4) % COLS as u32 },
                b => unsafe {
                    if CONTEXT.x == 0 {
                        TEXT_GRID[CONTEXT.y as usize].fill(b' ');
                    }
                    TEXT_GRID[CONTEXT.y as usize][CONTEXT.x as usize] = b;
                    CONTEXT.x += 1;
                },
            }
            REDRAW.store(true, Ordering::SeqCst);
            0
        }

        pub fn puts(s: &str) -> i32 {
            // print!("{}", s);
            for b in s.bytes() {
                Self::putc(b);
            }
            REDRAW.store(true, Ordering::SeqCst);
            0
        }

        pub fn clear() -> i32 {
            let _ = unsafe {
                DISPLAY
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .clear(Rgb565::from(RawU16::new(0x0000)))
            };
            REDRAW.store(true, Ordering::SeqCst);
            0
        }

        pub fn getkc() -> i32 {
            unsafe { KC_RECEIVER.as_mut() }
                .and_then(|r| match r.try_recv() {
                    Ok(kc) => Some(kc),
                    _ => None,
                })
                .unwrap_or(0)
        }
    }

    impl core::fmt::Write for Host {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            Self::puts(s);
            Ok(())
        }
    }
}
