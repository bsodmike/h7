#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use {
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
        .bypass_hse()
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
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
    let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);
    let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);

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

    // Display
    // TODO - LTDC
    // let mut display = stm32h7xx_hal::ltdc::Ltdc::new(dp.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
    // let display_config = DisplayConfiguration {
    //     active_width: 640,
    //     active_height: 480,
    //     h_back_porch: 0,
    //     h_front_porch: 0,
    //     v_back_porch: 0,
    //     v_front_porch: 0,
    //     h_sync: 0,
    //     v_sync: 0,

    //     /// horizontal synchronization: `false`: active low, `true`: active high
    //     h_sync_pol: false,
    //     /// vertical synchronization: `false`: active low, `true`: active high
    //     v_sync_pol: false,
    //     /// data enable: `false`: active low, `true`: active high
    //     not_data_enable_pol: false,
    //     /// pixel_clock: `false`: active low, `true`: active high
    //     pixel_clock_pol: false,
    // };
    // display.init(display_config);
    // let mut layer1 = display.split();
    // // let framebuf = alloc::boxed::Box::new([0u8; 640 * 480]);
    // let framebuf = [0u8; 640 * 480];
    // layer1.enable(framebuf.as_ptr(), PixelFormat::L8);
    // layer1.swap_framebuffer(framebuf.as_ptr());

    // USB Keyboard
    let usb2 = USB2::new(
        dp.OTG2_HS_GLOBAL,
        dp.OTG2_HS_DEVICE,
        dp.OTG2_HS_PWRCLK,
        gpioa.pa11.into_alternate_af10(),
        gpioa.pa12.into_alternate_af10(),
        ccdr.peripheral.USB2OTG,
        &ccdr.clocks,
    );
    let usb2_bus = UsbBus::new(usb2, &mut EP_MEMORY);
    // usb2_bus.interrupt(64, 100);

    cp.NVIC
        .set_priority(stm32h7xx_hal::pac::interrupt::OTG_HS, 1);
    cortex_m::peripheral::NVIC::unmask(stm32h7xx_hal::pac::interrupt::OTG_HS);

    // usb2_bus.

    // Crypto chip
    // TODO
    // https://crates.io/crates/Rusty_CryptoAuthLib

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
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:?}", ef);
}

#[cfg(not(feature = "semihosting"))]
#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
