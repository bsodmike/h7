#![feature(
    generic_const_exprs,
    const_trait_impl,
    duration_constants,
    const_mut_refs
)]

use {
    embedded_graphics::{
        mono_font::{MonoFont, MonoTextStyle},
        pixelcolor::Rgb565,
        prelude::*,
        primitives::{PrimitiveStyle, Rectangle, StyledDrawable},
        text::{renderer::TextRenderer, Text},
    },
    h7_display::{FrameBuffer, H7Display},
    sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum},
    std::{
        alloc::{alloc, dealloc, Layout},
        mem::{align_of, size_of},
        time::{Duration, Instant},
    },
};

mod input;
mod utils;

const FPS_TARGET: u32 = 60;
const WIDTH: usize = 1280;
const HEIGHT: usize = 768;
type PixelColor = Rgb565;

const BACKGROUND_COLOR: PixelColor = PixelColor::BLACK;
const TEXT_COLOR: PixelColor = PixelColor::CSS_WHEAT;

// const FONT: MonoFont = profont::PROFONT_18_POINT;
const FONT: MonoFont = ibm437::IBM437_9X14_REGULAR;

fn main() -> Result<(), String> {
    let lib = std::env::args()
        .nth(1)
        .map(|path| unsafe { libloading::Library::new(path) });
    let func = match lib {
        Some(Ok(ref lib)) => match unsafe { lib.get::<h7_api::AppEntryPoint>(b"entry_point") } {
            Ok(func) => Some(func),
            Err(e) => return Err(e.to_string()),
        },
        Some(Err(e)) => return Err(e.to_string()),
        None => None,
    };

    // let api = h7_api::H7Api {
    //     alloc:
    // };

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(env!("CARGO_PKG_NAME"), WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .vulkan()
        // .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let textute_creator = canvas.texture_creator();
    let mut texture = textute_creator
        .create_texture_streaming(PixelFormatEnum::RGB565, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let (vram_ptr, vram_layout, front_buffer, back_buffer) = unsafe {
        const FBSZ: usize = size_of::<FrameBuffer<PixelColor, WIDTH, HEIGHT>>();
        const FBAL: usize = align_of::<FrameBuffer<PixelColor, WIDTH, HEIGHT>>();
        // This assertion makes sure that consecutive framebuffers will be properly aligned.
        assert_eq!(FBSZ % FBAL, 0);
        let layout = Layout::from_size_align(FBSZ * 2, FBAL).unwrap();
        let vram_ptr = alloc(layout);
        let front_buffer = &mut *(vram_ptr as *mut _);
        let back_buffer = &mut *(vram_ptr.add(FBSZ) as *mut _);
        (vram_ptr, layout, front_buffer, back_buffer)
    };
    // sz_al_of!(FrameBuffer<COLOR, WIDTH, HEIGHT>);
    // sz_al_of!(H7Display::<COLOR, WIDTH, HEIGHT>);
    println!("vram_layout: {vram_layout:?}");
    let mut display = H7Display::<PixelColor, WIDTH, HEIGHT>::new(front_buffer, back_buffer);
    let mut input_buffer = input::InputBuffer::<142>::new();

    'running: loop {
        let sof = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // Event::KeyDown {
                //     keycode: Some(Keycode::S),
                //     ..
                // } => {
                //     utils::timer("Swap buffers", || {
                //         display.swap_buffers();
                //     });
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::Num1),
                //     ..
                // } => {
                //     // let w = display.width();
                //     // let back = display.back_buffer_mut();
                //     // back[0..(w * 20)].fill(PixelColor::RED);
                //     let _ = input_buffer.push_str("gqÅÄÖ_^");
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::Num2),
                //     ..
                // } => {
                //     let w = display.width();
                //     let back = display.back_buffer_mut();
                //     back[(w * 20)..(w * 40)].fill(PixelColor::GREEN);
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::Num3),
                //     ..
                // } => {
                //     let w = display.width();
                //     let back = display.back_buffer_mut();
                //     back[(w * 40)..(w * 60)].fill(PixelColor::BLUE);
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::R),
                //     ..
                // } => {
                //     display
                //         .fill_solid(
                //             &Rectangle::new(Point::new(0, 0), Size::new(100, 100)),
                //             PixelColor::CSS_CYAN,
                //         )
                //         .unwrap();
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::C),
                //     ..
                // } => {
                //     utils::timer("Clear", || {
                //         display.clear(PixelColor::BLACK).unwrap();
                //     });
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::L),
                //     ..
                // } => {
                //     utils::timer("Lines", || {
                //         let line_height = 32;
                //         for line in 0..(display.height() / line_height) {
                //             let color = if line % 2 == 0 {
                //                 PixelColor::CSS_GRAY
                //             } else {
                //                 PixelColor::CSS_LIGHT_GRAY
                //             };
                //             display
                //                 .fill_solid(
                //                     &Rectangle::new(
                //                         Point::new(0, (line * line_height) as i32),
                //                         Size::new(display.width() as u32, line_height as u32),
                //                     ),
                //                     color,
                //                 )
                //                 .unwrap();
                //         }
                //     });
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::T),
                //     ..
                // } => {
                //     utils::timer("Text", || {
                //         let text_style =
                //             MonoTextStyle::new(&profont::PROFONT_24_POINT, PixelColor::BLACK);
                //         let line_height = 32;
                //         for line in 0..(display.height() / line_height) {
                //             Text::new(
                //                 &format!("{line}"),
                //                 Point::new(50, ((line * line_height) + line_height - 8) as i32),
                //                 text_style,
                //             )
                //             .draw(&mut display)
                //             .unwrap();
                //         }
                //     });
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::Up),
                //     ..
                // } => {
                //     utils::timer("Scroll up", || display.scroll(-32, PixelColor::GREEN));
                // }
                // Event::KeyDown {
                //     keycode: Some(Keycode::Down),
                //     ..
                // } => {
                //     utils::timer("Scroll down", || display.scroll(32, PixelColor::RED));
                // }
                Event::KeyDown {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat,
                } => {
                    // println!("{keycode:?}");
                    if let Some(kc) = keycode.map(|kc| kc as i32) {
                        const N0: i32 = b'0' as i32;
                        const N9: i32 = b'9' as i32;
                        const A: i32 = b'a' as i32;
                        const Z: i32 = b'z' as i32;
                        match kc {
                            A..=Z | N0..=N9 => {
                                let mut c = kc as u8 as char;
                                if keymod.intersects(
                                    sdl2::keyboard::Mod::CAPSMOD
                                        | sdl2::keyboard::Mod::RSHIFTMOD
                                        | sdl2::keyboard::Mod::LSHIFTMOD,
                                ) {
                                    c = c.to_ascii_uppercase();
                                }

                                let _ = input_buffer.push(c);
                                println!("S: {}", input_buffer.as_str());
                            }
                            32 => {
                                let _ = input_buffer.push(' ');
                                println!("S: {}", input_buffer.as_str());
                            }
                            13 => {
                                input_buffer.clear();
                                println!("S: {}", input_buffer.as_str());
                                let _ = input_buffer.push_str("[root@h7] ");
                                utils::timer("Scroll down", || {
                                    display
                                        .scroll(FONT.character_size.height as i32, BACKGROUND_COLOR)
                                });
                            }
                            8 => {
                                input_buffer.pop();
                                println!("S: {}", input_buffer.as_str());
                            }
                            _ => {}
                        }
                        utils::timer("Text", || {
                            let text_style = MonoTextStyle::new(&FONT, TEXT_COLOR);
                            let line_height = text_style.line_height() as i32;

                            let y = HEIGHT as i32 - line_height;
                            Rectangle::new(
                                Point::new(0, y),
                                Size::new(WIDTH as u32, line_height as u32),
                            )
                            .draw_styled(&PrimitiveStyle::with_fill(BACKGROUND_COLOR), &mut display)
                            .unwrap();

                            let offset = text_style.line_height() - text_style.font.baseline;
                            Text::new(
                                input_buffer.as_str(),
                                // Point::new(0, y + (line_height / 2) + offset),
                                Point::new(0, y + line_height - offset as i32),
                                text_style,
                            )
                            .draw(&mut display)
                            .unwrap();
                        });
                    }
                }
                _ => {}
            }
        }

        // Copy our front buffer to the SDL texture and commit
        // some unsafe crimes while we're at it.
        let front = display.front_buffer();

        texture
            .with_lock(None, |buffer, _| {
                buffer.copy_from_slice(unsafe {
                    core::slice::from_raw_parts(
                        front.as_ptr() as *const u8,
                        front.len() * size_of::<PixelColor>(),
                    )
                });
            })
            .unwrap();

        // Copy SDL texture to canvas
        canvas.copy(&texture, None, None).unwrap();

        // Swap sdl2 buffers
        canvas.present();

        // Swap our own buffer
        display.swap_buffers();

        let diff = Instant::now() - sof;
        // let fps = 1_000_000f64 / diff.as_micros() as f64;
        // eprintln!("FT: {:.02}ms, FPS: {fps:.02}", diff.as_secs_f64() * 1000.0);

        if diff < Duration::SECOND / FPS_TARGET {
            std::thread::sleep((Duration::SECOND / FPS_TARGET) - diff);
        }

        // let diff = Instant::now() - sof;
        // let fps = 1_000_000f64 / diff.as_micros() as f64;
        // eprintln!("FT: {:.02}ms, FPS: {fps:.02}", diff.as_secs_f64() * 1000.0);
    }

    unsafe { dealloc(vram_ptr, vram_layout) };

    Ok(())
}
