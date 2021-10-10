#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use {
    anx7625::Anx7625,
    cortex_m_alloc::CortexMHeap,
    // embedded_display_controller::{
    //     DisplayConfiguration, DisplayController, DisplayControllerLayer, PixelFormat,
    // },
    log,
    stm32h7xx_hal::{
        hal::digital::v2::{OutputPin, ToggleableOutputPin},
        interrupt, pac,
        prelude::*,
        rcc,
        usb_hs::{UsbBus, USB2},
    },
    // Rusty_CryptoAuthLib::ATECC608A,
};

#[cfg(feature = "semihosting")]
mod logger;
mod sdram;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// static mut FRAME_BUF: Option<alloc::boxed::Box<[u16; 1024 * 768]>> = None;
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    #[cfg(feature = "semihosting")]
    logger::init();

    // Hi
    log::info!("Hello, world!");

    // Get peripherals
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Ah, yes
    // Copy the PWR CR3 power register value from a working Arduino sketch and write the value
    // directly since I cannot for the life of me figure out how to get it working with the
    // provided power configuration methods.
    // This is obviously not the *proper way* to do things but it works. The orange DL2 LED
    // is still lit meaning something is indeed not configured correctly.
    core::ptr::write_volatile(0x5802480c as *mut u32, 0b00000101000000010000000001010110);

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    // Constrain and Freeze clock
    // let rcc = dp.RCC.constrain();
    let mut ccdr = dp
        .RCC
        .constrain()
        // .bypass_hse()
        .sys_ck(480.mhz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .pll2_strategy(rcc::PllConfigStrategy::Iterative)
        .pll3_strategy(rcc::PllConfigStrategy::Iterative)
        // .pll3_p_ck(100.mhz())
        // .pll3_r_ck(26.mhz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // USB Clock
    let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
    ccdr.peripheral
        .kernel_usb_clk_mux(rcc::rec::UsbClkSel::HSI48);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // GPIO
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
    let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);
    let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);
    let gpioj = dp.GPIOJ.split(ccdr.peripheral.GPIOJ);

    // Configure SDRAM pins
    let sdram_pins = fmc_pins! {
        // A0-A12
        gpiof.pf0, gpiof.pf1, gpiof.pf2, gpiof.pf3,
        gpiof.pf4, gpiof.pf5, gpiof.pf12, gpiof.pf13,
        gpiof.pf14, gpiof.pf15, gpiog.pg0, gpiog.pg1,
        gpiog.pg2,
        // BA0-BA1
        gpiog.pg4, gpiog.pg5,
        // D0-D15
        gpiod.pd14, gpiod.pd15, gpiod.pd0, gpiod.pd1,
        gpioe.pe7, gpioe.pe8, gpioe.pe9, gpioe.pe10,
        gpioe.pe11, gpioe.pe12, gpioe.pe13, gpioe.pe14,
        gpioe.pe15, gpiod.pd8, gpiod.pd9, gpiod.pd10,
        // NBL0 - NBL1
        gpioe.pe0, gpioe.pe1,
        gpioh.ph2, // SDCKE1
        gpiog.pg8, // SDCLK
        gpiog.pg15, // SDCAS
        gpioh.ph3, // SDNE1 (!CS)
        gpiof.pf11, // SDRAS
        gpioh.ph5 // SDNWE
    };

    // Init SDRAM
    sdram::configure(&cp.MPU, &cp.SCB);
    let sdram_ptr = dp
        .FMC
        .sdram(
            sdram_pins,
            stm32_fmc::devices::as4c4m16sa_6::As4c4m16sa {},
            ccdr.peripheral.FMC,
            &ccdr.clocks,
        )
        .init(&mut delay);

    // Configure allocator
    ALLOCATOR.init(sdram_ptr as usize, sdram::SDRAM_SIZE);

    // Enable osc?
    let mut oscen = gpioh.ph1.into_push_pull_output();
    delay.delay_ms(10u32);
    oscen.set_high().unwrap();
    delay.delay_ms(1000u32);

    // Display
    // TODO - LTDC
    let mut internal_i2c = dp.I2C1.i2c(
        (
            gpiob.pb6.into_alternate_af4().set_open_drain(),
            gpiob.pb7.into_alternate_af4().set_open_drain(),
        ),
        400.khz(),
        ccdr.peripheral.I2C1,
        &ccdr.clocks,
    );
    internal_i2c.write(0x08, &[0x42, 0x01]).unwrap(); // void fixup3V1Rail()???
    internal_i2c.write(0x08, &[0x52, 0x09]).unwrap(); // LDO3 to 1.2V
    internal_i2c.write(0x08, &[0x53, 0x0f]).unwrap();
    internal_i2c.write(0x08, &[0x3b, 0x0f]).unwrap(); // SW2 to 3.3V (SW2_VOLT)
    internal_i2c.write(0x08, &[0x35, 0x0f]).unwrap(); // SW1 to 3.0V (SW1_VOLT)

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

    // USB Keyboard
    // let usb2 = USB2::new(
    //     dp.OTG2_HS_GLOBAL,
    //     dp.OTG2_HS_DEVICE,
    //     dp.OTG2_HS_PWRCLK,
    //     gpioa.pa11.into_alternate_af10(),
    //     gpioa.pa12.into_alternate_af10(),
    //     ccdr.peripheral.USB2OTG,
    //     &ccdr.clocks,
    // );
    // let usb2_bus = UsbBus::new(usb2, &mut EP_MEMORY);
    // usb2_bus.interrupt(64, 100);

    cp.NVIC
        .set_priority(stm32h7xx_hal::pac::interrupt::OTG_HS, 1);
    cortex_m::peripheral::NVIC::unmask(stm32h7xx_hal::pac::interrupt::OTG_HS);

    // usb2_bus.

    // Crypto chip
    // TODO
    // https://crates.io/crates/Rusty_CryptoAuthLib
    // let crypto_delay = stm32h7xx_hal::delay::DelayFromCountDownTimer::new(dp.TIM2.timer(
    //     100.ms(),
    //     ccdr.peripheral.TIM2,
    //     &mut ccdr.clocks,
    // ));
    // let crypto_timer = dp
    //     .TIM3
    //     .timer(100.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
    // let crypto = ATECC608A::new(internal_i2c, crypto_delay, crypto_timer);
    let atecc608x_info = atecc608x::Atecc608x::info(&mut internal_i2c, &mut delay);
    log::info!("{:?}", atecc608x_info);

    // Configure PK5, PK6, PK7 as output.
    // let mut led_r = gpiok.pk5.into_push_pull_output();
    let mut led_g = gpiok.pk6.into_push_pull_output();
    // let mut led_b = gpiok.pk7.into_push_pull_output();

    loop {
        led_g.toggle().unwrap();
        delay.delay_ms(500_u16);
    }
}

#[interrupt]
fn OTG_HS() {
    log::info!("USB");
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("{:?}", layout)
}

#[cortex_m_rt::exception]
unsafe fn DefaultHandler(irqn: i16) -> ! {
    panic!("IRQn {:?}", irqn);
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:?}", ef);
}

#[cfg(not(feature = "semihosting"))]
#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
