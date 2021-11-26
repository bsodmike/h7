use {
    display::*,
    embedded_graphics::{
        mono_font::iso_8859_10::FONT_10X20 as ISO_FONT_10X20,
        pixelcolor::{raw::RawU16, Rgb565},
        prelude::*,
    },
    embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window},
};

mod display;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

const INVERT_COLORS: bool = false;
const TEXT_COLOR: u16 = 0xD634;
const BACKGROUND_COLOR: u16 = 0x18E3;

// const FONT_SETTINGS: fontdue::FontSettings = fontdue::FontSettings {
//     collection_index: 0,
//     scale: 32.0,
// };

// const SOURCE_CODE_PRO_REGULAR_BYTES: &[u8] = include_bytes!("../fonts/SourceCodePro-Regular.ttf");
// const IBM_PLEX_MONO_BYTES: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
// const IBM_VGA_9X16_2X: &[u8] = include_bytes!("../fonts/IBM_VGA_9x16-2x.ttf");

const TEST_STRS: &[&str] = &[
    "----------------------- MCU ------------------------",
    "MCU                         STM32H747",
    "Unique ID                   4C0043001251303130373836",
    "----------------------- CPU ------------------------",
    "Core                        Cortex-M7F",
    "Core frequency              480MHz",
    "Core temperature            38.7°C",
    "Cycle count                 1366884323",
    "Instruction cache enabled   true",
    "Data cache enabled          true",
    "---------------------- Memory ----------------------",
    "Internal RAM                512KiB",
    "Internal FLASH              2048KiB",
    "External SDRAM              8192KiB",
    "External FLASH              unavailable",
    "--------------------- SD Card ----------------------",
    "SD Card mounted             false",
    "Size                        Not Mounted",
    "------------------------ OS ------------------------",
    "Version                     heads/rust-0-ge0051a1-dirty",
    "Debug                       false",
    "Compiled                    Fri Nov 26 01:43:45 2021",
    "-------------------- Date/Time ---------------------",
    "Fri Nov 26 05:42:46 2021",
];

fn main() -> Result<(), core::convert::Infallible> {
    // let source_code_pro_regular =
    //     fontdue::Font::from_bytes(SOURCE_CODE_PRO_REGULAR_BYTES, FONT_SETTINGS).unwrap();
    // let ibm_plex_mono = fontdue::Font::from_bytes(IBM_PLEX_MONO_BYTES, FONT_SETTINGS).unwrap();
    // let ibm_vga_9x16_2x = fontdue::Font::from_bytes(IBM_VGA_9X16_2X, FONT_SETTINGS).unwrap();

    let text_color = Rgb565::from(RawU16::new(if INVERT_COLORS {
        !TEXT_COLOR
    } else {
        TEXT_COLOR
    }));
    let backgroud_color = Rgb565::from(RawU16::new(if INVERT_COLORS {
        !BACKGROUND_COLOR
    } else {
        BACKGROUND_COLOR
    }));

    let mut disp = Display::new(SimulatorDisplay::<Rgb565>::new(Size::new(WIDTH, HEIGHT)));

    disp.clear(backgroud_color)?;

    let mut ypos = 16u32;

    ypos += disp
        .draw_text(
            &format!("Colors inverted: {}", INVERT_COLORS),
            &ISO_FONT_10X20,
            text_color,
            XPos::Right(8),
            YPos::Top(ypos as i32),
        )?
        .height;

    ypos += disp
        .draw_text(
            &format!("Text color: 0x{:04x}", TEXT_COLOR),
            &ISO_FONT_10X20,
            text_color,
            XPos::Right(8),
            YPos::Top(ypos as i32),
        )?
        .height;

    disp.draw_text(
        &format!("Background color: 0x{:04x}", BACKGROUND_COLOR),
        &ISO_FONT_10X20,
        text_color,
        XPos::Right(8),
        YPos::Top(ypos as i32),
    )?;

    ypos = 16u32;
    for line in TEST_STRS {
        // ypos += disp
        //     .draw_text_fontdue(
        //         &[(
        //             // "Almost before we knew it, we had left the ground.",
        //             line,
        //             &source_code_pro_regular,
        //         )],
        //         24.0,
        //         text_color,
        //         48,
        //         XPos::Left(8),
        //         YPos::Top(ypos as i32),
        //     )?
        //     .height;
        ypos += disp
            .draw_text(
                line,
                &ISO_FONT_10X20,
                text_color,
                XPos::Left(8),
                YPos::Top(ypos as i32),
            )?
            .height;
    }

    let output_settings = OutputSettingsBuilder::new().build();

    Window::new(env!("CARGO_PKG_NAME"), &output_settings).show_static(&disp.display);

    Ok(())
}
