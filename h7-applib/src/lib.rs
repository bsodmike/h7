#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
mod display;

#[cfg(feature = "c-api")]
pub mod c_api;

#[cfg(feature = "alloc")]
extern crate alloc;

pub struct Host;

#[cfg(feature = "alloc")]
mod h7_alloc {
    pub struct H7Allocator;

    impl H7Allocator {
        pub(crate) unsafe fn dealloc_all(&self) {
            // TODO
        }
    }

    unsafe impl core::alloc::GlobalAlloc for H7Allocator {
        #[inline(always)]
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            super::Host::alloc(layout)
        }

        #[inline(always)]
        unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
            super::Host::free(ptr, layout)
        }
    }
}

#[cfg(feature = "alloc")]
pub use h7_alloc::H7Allocator;

/// Implementation used when building code for the H7
#[cfg(target_os = "none")]
pub mod target {

    use {
        super::*,
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
    pub extern "C" fn entry_point(table: *const H7Api) -> i32 {
        // Turn the pointer into a reference and store in a static.
        unsafe {
            API_POINTER.write(&*table);
        };

        extern "C" {
            fn h7_main() -> i32;
        }
        // Call the user application
        let ret = unsafe { h7_main() };

        // Free leaked memory
        #[cfg(feature = "alloc")]
        unsafe {
            H7Allocator.dealloc_all()
        };

        ret
    }

    #[inline(always)]
    fn get_api() -> &'static H7Api {
        unsafe { API_POINTER.assume_init() }
    }

    impl Host {
        pub(crate) fn alloc(layout: core::alloc::Layout) -> *mut u8 {
            todo!()
        }

        pub(crate) fn free(ptr: *mut u8, layout: core::alloc::Layout) {
            todo!()
        }

        pub fn panic(msg: &str) -> ! {
            // todo!()
            loop {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }

        pub fn delay(ms: u32) {}

        #[inline(always)]
        pub fn puts(s: &str) -> i32 {
            (get_api().puts)(s.as_ptr(), s.len())
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

    #[cfg(feature = "default-panic-handler")]
    #[inline(never)]
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        Host::panic("User application paniced")
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
            alloc::GlobalAlloc,
            sync::{
                atomic::{AtomicBool, Ordering},
                mpsc::{channel, Receiver},
                Arc, Mutex,
            },
            time::{Duration, Instant},
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

    static mut TEXT_GRID: [[(char, u16); COLS]; ROWS] = [[(b' ' as char, 0xffff); COLS]; ROWS];
    static mut KC_RECEIVER: Option<Receiver<i32>> = None;
    static mut CONTEXT: Context = Context {
        x: 0,
        y: (ROWS - 1) as u32,
    };
    static mut DISPLAY: Option<Arc<Mutex<Display<SimulatorDisplay<Rgb565>>>>> = None;
    static REDRAW: AtomicBool = AtomicBool::new(true);

    fn draw_text_grid() {
        let mut lock = unsafe { DISPLAY.as_mut().unwrap().lock().unwrap() };

        for (y, line) in unsafe { TEXT_GRID }.iter().enumerate() {
            for (x, (c, color)) in line.iter().enumerate() {
                let mut tmp = [0u8; 4];
                let _ = lock.draw_text(
                    c.encode_utf8(&mut tmp),
                    &ISO_FONT_10X20,
                    Rgb565::from(RawU16::new(*color)),
                    XPos::Absolute((x * GLYPH_WIDTH) as i32),
                    YPos::Absolute((y * GLYPH_HEIGHT) as i32 + 18),
                );
            }
        }
    }

    fn scroll(n: usize) {
        for _ in 0..n {
            for line in 0..unsafe { TEXT_GRID }.len() - 1 {
                unsafe {
                    core::mem::swap(&mut TEXT_GRID[line], &mut TEXT_GRID[line + 1]);
                }
            }
        }
    }

    impl Host {
        pub fn init() {
            let (tx, rx) = channel();
            unsafe {
                println!("Text buffer: {}", core::mem::size_of_val(&TEXT_GRID));
                KC_RECEIVER = Some(rx);
                DISPLAY = Some(Arc::new(Mutex::new(Display::new(
                    SimulatorDisplay::<Rgb565>::new(Size::new(WIDTH, HEIGHT)),
                ))))
            };

            let disp = Arc::clone(unsafe { DISPLAY.as_ref().unwrap() });
            std::thread::spawn(move || {
                let output_settings = OutputSettingsBuilder::new().build();
                let mut w = Window::new(env!("CARGO_PKG_NAME"), &output_settings);

                let mut kb = pc_keyboard::Keyboard::new(
                    pc_keyboard::layouts::Us104Key,
                    pc_keyboard::ScancodeSet2,
                    pc_keyboard::HandleControl::Ignore,
                );

                loop {
                    if REDRAW.load(Ordering::SeqCst) {
                        println!("REDRAW!");
                        draw_text_grid();
                        if let Ok(lock) = disp.try_lock() {
                            w.update(&lock.display);
                            REDRAW.store(false, Ordering::SeqCst);
                        }
                    }
                    for e in w.events() {
                        match e {
                            SimulatorEvent::Quit => std::process::exit(1),
                            SimulatorEvent::KeyUp { keycode, .. } => {
                                if let Some(k) = kb.process_keyevent(pc_keyboard::KeyEvent {
                                    code: convert_keycode(keycode),
                                    state: pc_keyboard::KeyState::Up,
                                }) {
                                    println!("{:?}", k);
                                }
                            }
                            SimulatorEvent::KeyDown { keycode, .. } => {
                                if let Some(k) = kb.process_keyevent(pc_keyboard::KeyEvent {
                                    code: convert_keycode(keycode),
                                    state: pc_keyboard::KeyState::Down,
                                }) {
                                    println!("{:?}", k);
                                }
                                // HOME key
                                if keycode as i32 == 1073741898 {
                                    REDRAW.store(true, Ordering::SeqCst);
                                }
                                drop(tx.send(keycode as i32));
                            }
                            _ => {}
                        }
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            });
        }

        #[inline(always)]
        pub(crate) fn alloc(layout: core::alloc::Layout) -> *mut u8 {
            unsafe { std::alloc::System.alloc(layout) }
        }

        #[inline(always)]
        pub(crate) fn free(ptr: *mut u8, layout: core::alloc::Layout) {
            unsafe { std::alloc::System.dealloc(ptr, layout) }
        }

        pub fn panic(msg: &str) -> ! {
            panic!("{}", msg)
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
                    scroll(1);
                    TEXT_GRID[(ROWS - 1) as usize].fill((b' ' as char, 0xffff));
                },
                b'\t' => unsafe { CONTEXT.x = (CONTEXT.x + 4) % COLS as u32 },
                b => unsafe {
                    TEXT_GRID[CONTEXT.y as usize][CONTEXT.x as usize].0 = b as char;
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

    fn convert_keycode(sdl_kc: embedded_graphics_simulator::sdl2::Keycode) -> pc_keyboard::KeyCode {
        use embedded_graphics_simulator::sdl2 as sdl;
        match sdl_kc {
            sdl::Keycode::Backspace => pc_keyboard::KeyCode::Backspace,
            sdl::Keycode::Tab => pc_keyboard::KeyCode::Tab,
            sdl::Keycode::Return => pc_keyboard::KeyCode::Enter,
            sdl::Keycode::Escape => pc_keyboard::KeyCode::Escape,
            sdl::Keycode::Space => pc_keyboard::KeyCode::Spacebar,
            sdl::Keycode::Exclaim => todo!(),
            sdl::Keycode::Quotedbl => todo!(),
            sdl::Keycode::Hash => todo!(),
            sdl::Keycode::Dollar => pc_keyboard::KeyCode::A,
            sdl::Keycode::Percent => pc_keyboard::KeyCode::A,
            sdl::Keycode::Ampersand => pc_keyboard::KeyCode::A,
            sdl::Keycode::Quote => pc_keyboard::KeyCode::A,
            sdl::Keycode::LeftParen => pc_keyboard::KeyCode::A,
            sdl::Keycode::RightParen => pc_keyboard::KeyCode::A,
            sdl::Keycode::Asterisk => pc_keyboard::KeyCode::A,
            sdl::Keycode::Plus => pc_keyboard::KeyCode::A,
            sdl::Keycode::Comma => pc_keyboard::KeyCode::A,
            sdl::Keycode::Minus => pc_keyboard::KeyCode::A,
            sdl::Keycode::Period => pc_keyboard::KeyCode::A,
            sdl::Keycode::Slash => pc_keyboard::KeyCode::A,
            sdl::Keycode::Num0 => pc_keyboard::KeyCode::Numpad0,
            sdl::Keycode::Num1 => pc_keyboard::KeyCode::Numpad1,
            sdl::Keycode::Num2 => pc_keyboard::KeyCode::Numpad2,
            sdl::Keycode::Num3 => pc_keyboard::KeyCode::Numpad3,
            sdl::Keycode::Num4 => pc_keyboard::KeyCode::Numpad4,
            sdl::Keycode::Num5 => pc_keyboard::KeyCode::Numpad5,
            sdl::Keycode::Num6 => pc_keyboard::KeyCode::Numpad6,
            sdl::Keycode::Num7 => pc_keyboard::KeyCode::Numpad7,
            sdl::Keycode::Num8 => pc_keyboard::KeyCode::Numpad8,
            sdl::Keycode::Num9 => pc_keyboard::KeyCode::Numpad9,
            sdl::Keycode::Colon => pc_keyboard::KeyCode::A,
            sdl::Keycode::Semicolon => pc_keyboard::KeyCode::A,
            sdl::Keycode::Less => pc_keyboard::KeyCode::A,
            sdl::Keycode::Equals => pc_keyboard::KeyCode::A,
            sdl::Keycode::Greater => pc_keyboard::KeyCode::A,
            sdl::Keycode::Question => pc_keyboard::KeyCode::A,
            sdl::Keycode::At => pc_keyboard::KeyCode::A,
            sdl::Keycode::LeftBracket => pc_keyboard::KeyCode::A,
            sdl::Keycode::Backslash => pc_keyboard::KeyCode::A,
            sdl::Keycode::RightBracket => pc_keyboard::KeyCode::A,
            sdl::Keycode::Caret => pc_keyboard::KeyCode::A,
            sdl::Keycode::Underscore => pc_keyboard::KeyCode::A,
            sdl::Keycode::Backquote => pc_keyboard::KeyCode::A,
            sdl::Keycode::A => pc_keyboard::KeyCode::A,
            sdl::Keycode::B => pc_keyboard::KeyCode::B,
            sdl::Keycode::C => pc_keyboard::KeyCode::C,
            sdl::Keycode::D => pc_keyboard::KeyCode::D,
            sdl::Keycode::E => pc_keyboard::KeyCode::E,
            sdl::Keycode::F => pc_keyboard::KeyCode::F,
            sdl::Keycode::G => pc_keyboard::KeyCode::G,
            sdl::Keycode::H => pc_keyboard::KeyCode::H,
            sdl::Keycode::I => pc_keyboard::KeyCode::I,
            sdl::Keycode::J => pc_keyboard::KeyCode::J,
            sdl::Keycode::K => pc_keyboard::KeyCode::K,
            sdl::Keycode::L => pc_keyboard::KeyCode::L,
            sdl::Keycode::M => pc_keyboard::KeyCode::M,
            sdl::Keycode::N => pc_keyboard::KeyCode::N,
            sdl::Keycode::O => pc_keyboard::KeyCode::O,
            sdl::Keycode::P => pc_keyboard::KeyCode::P,
            sdl::Keycode::Q => pc_keyboard::KeyCode::Q,
            sdl::Keycode::R => pc_keyboard::KeyCode::R,
            sdl::Keycode::S => pc_keyboard::KeyCode::S,
            sdl::Keycode::T => pc_keyboard::KeyCode::T,
            sdl::Keycode::U => pc_keyboard::KeyCode::U,
            sdl::Keycode::V => pc_keyboard::KeyCode::V,
            sdl::Keycode::W => pc_keyboard::KeyCode::W,
            sdl::Keycode::X => pc_keyboard::KeyCode::X,
            sdl::Keycode::Y => pc_keyboard::KeyCode::Y,
            sdl::Keycode::Z => pc_keyboard::KeyCode::Z,
            sdl::Keycode::Delete => pc_keyboard::KeyCode::Delete,
            sdl::Keycode::CapsLock => pc_keyboard::KeyCode::CapsLock,
            sdl::Keycode::F1 => pc_keyboard::KeyCode::F1,
            sdl::Keycode::F2 => pc_keyboard::KeyCode::F2,
            sdl::Keycode::F3 => pc_keyboard::KeyCode::F3,
            sdl::Keycode::F4 => pc_keyboard::KeyCode::F4,
            sdl::Keycode::F5 => pc_keyboard::KeyCode::F5,
            sdl::Keycode::F6 => pc_keyboard::KeyCode::F6,
            sdl::Keycode::F7 => pc_keyboard::KeyCode::F7,
            sdl::Keycode::F8 => pc_keyboard::KeyCode::F8,
            sdl::Keycode::F9 => pc_keyboard::KeyCode::F9,
            sdl::Keycode::F10 => pc_keyboard::KeyCode::F10,
            sdl::Keycode::F11 => pc_keyboard::KeyCode::F11,
            sdl::Keycode::F12 => pc_keyboard::KeyCode::F12,
            sdl::Keycode::PrintScreen => pc_keyboard::KeyCode::PrintScreen,
            sdl::Keycode::ScrollLock => pc_keyboard::KeyCode::ScrollLock,
            sdl::Keycode::Pause => pc_keyboard::KeyCode::PauseBreak,
            sdl::Keycode::Insert => pc_keyboard::KeyCode::Insert,
            sdl::Keycode::Home => pc_keyboard::KeyCode::Home,
            sdl::Keycode::PageUp => pc_keyboard::KeyCode::PageUp,
            sdl::Keycode::End => pc_keyboard::KeyCode::End,
            sdl::Keycode::PageDown => pc_keyboard::KeyCode::PageDown,
            sdl::Keycode::Right => pc_keyboard::KeyCode::ArrowRight,
            sdl::Keycode::Left => pc_keyboard::KeyCode::ArrowLeft,
            sdl::Keycode::Down => pc_keyboard::KeyCode::ArrowDown,
            sdl::Keycode::Up => pc_keyboard::KeyCode::ArrowUp,
            sdl::Keycode::NumLockClear => todo!(),
            sdl::Keycode::KpDivide => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMultiply => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMinus => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpPlus => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpEnter => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp1 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp2 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp3 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp4 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp5 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp6 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp7 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp8 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp9 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp0 => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpPeriod => pc_keyboard::KeyCode::A,
            sdl::Keycode::Application => pc_keyboard::KeyCode::A,
            sdl::Keycode::Power => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpEquals => pc_keyboard::KeyCode::A,
            sdl::Keycode::F13 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F14 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F15 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F16 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F17 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F18 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F19 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F20 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F21 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F22 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F23 => pc_keyboard::KeyCode::A,
            sdl::Keycode::F24 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Execute => pc_keyboard::KeyCode::A,
            sdl::Keycode::Help => pc_keyboard::KeyCode::A,
            sdl::Keycode::Menu => pc_keyboard::KeyCode::A,
            sdl::Keycode::Select => pc_keyboard::KeyCode::A,
            sdl::Keycode::Stop => pc_keyboard::KeyCode::A,
            sdl::Keycode::Again => pc_keyboard::KeyCode::A,
            sdl::Keycode::Undo => pc_keyboard::KeyCode::A,
            sdl::Keycode::Cut => pc_keyboard::KeyCode::A,
            sdl::Keycode::Copy => pc_keyboard::KeyCode::A,
            sdl::Keycode::Paste => pc_keyboard::KeyCode::A,
            sdl::Keycode::Find => pc_keyboard::KeyCode::A,
            sdl::Keycode::Mute => pc_keyboard::KeyCode::A,
            sdl::Keycode::VolumeUp => pc_keyboard::KeyCode::A,
            sdl::Keycode::VolumeDown => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpComma => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpEqualsAS400 => pc_keyboard::KeyCode::A,
            sdl::Keycode::AltErase => pc_keyboard::KeyCode::A,
            sdl::Keycode::Sysreq => pc_keyboard::KeyCode::A,
            sdl::Keycode::Cancel => pc_keyboard::KeyCode::A,
            sdl::Keycode::Clear => pc_keyboard::KeyCode::A,
            sdl::Keycode::Prior => pc_keyboard::KeyCode::A,
            sdl::Keycode::Return2 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Separator => pc_keyboard::KeyCode::A,
            sdl::Keycode::Out => pc_keyboard::KeyCode::A,
            sdl::Keycode::Oper => pc_keyboard::KeyCode::A,
            sdl::Keycode::ClearAgain => pc_keyboard::KeyCode::A,
            sdl::Keycode::CrSel => pc_keyboard::KeyCode::A,
            sdl::Keycode::ExSel => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp00 => pc_keyboard::KeyCode::A,
            sdl::Keycode::Kp000 => pc_keyboard::KeyCode::A,
            sdl::Keycode::ThousandsSeparator => pc_keyboard::KeyCode::A,
            sdl::Keycode::DecimalSeparator => pc_keyboard::KeyCode::A,
            sdl::Keycode::CurrencyUnit => pc_keyboard::KeyCode::A,
            sdl::Keycode::CurrencySubUnit => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpLeftParen => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpRightParen => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpLeftBrace => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpRightBrace => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpTab => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpBackspace => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpA => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpB => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpC => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpD => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpE => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpF => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpXor => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpPower => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpPercent => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpLess => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpGreater => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpAmpersand => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpDblAmpersand => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpVerticalBar => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpDblVerticalBar => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpColon => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpHash => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpSpace => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpAt => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpExclam => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemStore => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemRecall => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemClear => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemAdd => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemSubtract => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemMultiply => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpMemDivide => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpPlusMinus => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpClear => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpClearEntry => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpBinary => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpOctal => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpDecimal => pc_keyboard::KeyCode::A,
            sdl::Keycode::KpHexadecimal => pc_keyboard::KeyCode::A,
            sdl::Keycode::LCtrl => pc_keyboard::KeyCode::ControlLeft,
            sdl::Keycode::LShift => pc_keyboard::KeyCode::ShiftLeft,
            sdl::Keycode::LAlt => pc_keyboard::KeyCode::AltLeft,
            sdl::Keycode::LGui => pc_keyboard::KeyCode::WindowsLeft,
            sdl::Keycode::RCtrl => pc_keyboard::KeyCode::ControlRight,
            sdl::Keycode::RShift => pc_keyboard::KeyCode::ShiftRight,
            sdl::Keycode::RAlt => pc_keyboard::KeyCode::AltRight,
            sdl::Keycode::RGui => pc_keyboard::KeyCode::WindowsRight,
            sdl::Keycode::Mode => pc_keyboard::KeyCode::A,
            sdl::Keycode::AudioNext => pc_keyboard::KeyCode::NextTrack,
            sdl::Keycode::AudioPrev => pc_keyboard::KeyCode::PrevTrack,
            sdl::Keycode::AudioStop => pc_keyboard::KeyCode::Stop,
            sdl::Keycode::AudioPlay => pc_keyboard::KeyCode::Play,
            sdl::Keycode::AudioMute => pc_keyboard::KeyCode::Mute,
            sdl::Keycode::MediaSelect => pc_keyboard::KeyCode::A,
            sdl::Keycode::Www => pc_keyboard::KeyCode::WWWHome,
            sdl::Keycode::Mail => pc_keyboard::KeyCode::A,
            sdl::Keycode::Calculator => pc_keyboard::KeyCode::Calculator,
            sdl::Keycode::Computer => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcSearch => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcHome => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcBack => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcForward => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcStop => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcRefresh => pc_keyboard::KeyCode::A,
            sdl::Keycode::AcBookmarks => pc_keyboard::KeyCode::A,
            sdl::Keycode::BrightnessDown => pc_keyboard::KeyCode::A,
            sdl::Keycode::BrightnessUp => pc_keyboard::KeyCode::A,
            sdl::Keycode::DisplaySwitch => pc_keyboard::KeyCode::A,
            sdl::Keycode::KbdIllumToggle => pc_keyboard::KeyCode::A,
            sdl::Keycode::KbdIllumDown => pc_keyboard::KeyCode::A,
            sdl::Keycode::KbdIllumUp => pc_keyboard::KeyCode::A,
            sdl::Keycode::Eject => pc_keyboard::KeyCode::A,
            sdl::Keycode::Sleep => pc_keyboard::KeyCode::A,
        }
    }
}
