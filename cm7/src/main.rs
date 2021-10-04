#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use {
    cortex_m_alloc::CortexMHeap,
    log,
    stm32h7xx_hal::{hal::digital::v2::OutputPin, pac, prelude::*, rcc},
};

#[cfg(feature = "semihosting")]
mod logger;
mod sdram;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// static mut FRAME_BUF: Option<alloc::boxed::Box<[u16; 1024 * 768]>> = None;

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    #[cfg(feature = "semihosting")]
    logger::init();

    // Hi
    log::info!("Hello, world!");

    // Get peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
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
    let ccdr = dp
        .RCC
        .constrain()
        .bypass_hse()
        .sys_ck(480.mhz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .freeze(pwrcfg, &dp.SYSCFG);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // GPIO
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

    // USB Keyboard
    // TODO

    // Crypto chip
    // TODO
    // https://crates.io/crates/Rusty_CryptoAuthLib

    // Configure PK5, PK6, PK7 as output.
    let mut led_r = gpiok.pk5.into_push_pull_output();
    let mut led_g = gpiok.pk6.into_push_pull_output();
    let mut led_b = gpiok.pk7.into_push_pull_output();

    loop {
        led_r.set_high().unwrap();
        delay.delay_ms(100_u16);
        led_g.set_high().unwrap();
        delay.delay_ms(100_u16);
        led_b.set_high().unwrap();
        delay.delay_ms(100_u16);

        led_r.set_low().unwrap();
        delay.delay_ms(100_u16);
        led_g.set_low().unwrap();
        delay.delay_ms(100_u16);
        led_b.set_low().unwrap();
        delay.delay_ms(100_u16);
    }
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
