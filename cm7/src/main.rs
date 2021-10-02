#![no_main]
#![no_std]

use {
    log,
    stm32h7xx_hal::{hal::digital::v2::OutputPin, pac, prelude::*},
};

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    log::info!("Hello, world!");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Constrain and Freeze power
    log::info!("Setup PWR");
    let pwr = dp.PWR.constrain();
    // let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    // Constrain and Freeze clock
    log::info!("Setup RCC");
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .bypass_hse()
        .sys_ck(360.mhz())
        // .pll1_q_ck(480.mhz())
        .freeze(pwrcfg, &dp.SYSCFG);

    let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);

    // Configure PK6 as output.
    let mut led = gpiok.pk6.into_push_pull_output();

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    loop {
        led.set_high().unwrap();
        delay.delay_ms(500_u16);

        led.set_low().unwrap();
        delay.delay_ms(500_u16);
    }
}

#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
