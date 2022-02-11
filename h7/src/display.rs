// let mut anx7625 = Anx7625::new(
//     // dp.I2C1.i2c(
//     //     (
//     //         gpiob.pb6.into_alternate_af4().set_open_drain(),
//     //         gpiob.pb7.into_alternate_af4().set_open_drain(),
//     //     ),
//     //     400.khz(),
//     //     ccdr.peripheral.I2C1,
//     //     &ccdr.clocks,
//     // ),
//     internal_i2c,
//     gpiok.pk2.into_push_pull_output(),
//     gpioj.pj3.into_push_pull_output(),
// );
// match anx7625.init(gpioj.pj6.into_push_pull_output(), &mut delay) {
//     Ok(_) => (),
//     Err(e) => panic!("{}", e),
// };
// let mut display = stm32h7xx_hal::ltdc::Ltdc::new(dp.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
// let display_config = DisplayConfiguration {
//     active_width: 1280,
//     active_height: 768,
//     h_back_porch: 120,
//     h_front_porch: 32,
//     v_back_porch: 10,
//     v_front_porch: 45,
//     h_sync: 20,
//     v_sync: 12,

//     /// horizontal synchronization: `false`: active low, `true`: active high
//     h_sync_pol: true,
//     /// vertical synchronization: `false`: active low, `true`: active high
//     v_sync_pol: true,
//     /// data enable: `false`: active low, `true`: active high
//     not_data_enable_pol: false,
//     /// pixel_clock: `false`: active low, `true`: active high
//     pixel_clock_pol: false,
// };
// display.init(display_config);
// let mut layer1 = display.split();
// // let framebuf = alloc::boxed::Box::new([0u8; 640 * 480]);
// let framebuf = [0u8; 1280 * 768];
// layer1.enable(framebuf.as_ptr(), PixelFormat::L8);
// layer1.swap_framebuffer(framebuf.as_ptr());

type Pixel = u16;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_HEIGHT: usize = 768;
pub const FRAME_BUF_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * core::mem::size_of::<Pixel>() * 2;

pub fn frame_buffer() -> &'static mut [u16] {
    todo!()
}

fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let rgb888 = u32::from_be_bytes([0, r, g, b]);
    let r16 = ((rgb888 & 0x000000F8) >> 3) as u16;
    let g16 = ((rgb888 & 0x0000FC00) >> 5) as u16;
    let b16 = ((rgb888 & 0x00F80000) >> 8) as u16;
    r16 | g16 | b16
}
