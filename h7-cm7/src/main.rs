#![no_main]
#![no_std]
#![feature(
    alloc_error_handler,
    const_for,
    const_mut_refs,
    array_chunks,
    generic_const_exprs
)]

// use embedded_display_controller::DisplayController;

extern crate alloc;

use {
    crate::utils::interrupt_free,
    // anx7625::Anx7625,
    chrono::{NaiveDate, Timelike},
    core::fmt::Write,
    embedded_display_controller::{
        DisplayConfiguration, DisplayController, DisplayControllerLayer, PixelFormat,
    },
    fugit::RateExtU32,
    led::Led,
    stm32h7xx_hal::{
        self as hal, adc,
        gpio::Speed,
        pac,
        prelude::*,
        rcc, rtc,
        usb_hs::{UsbBus, USB1_ULPI},
    },
    time::TimeSource,
};

mod app;
mod consts;
mod display;
mod dsi;
mod fs;
mod led;
mod logger;
mod mem;
#[cfg(not(feature = "semihosting"))]
mod panic;
mod pmic;
mod system;
mod terminal;
mod time;
mod utils;

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    logger::init();
    // log::info!("Booting up...");

    // Get peripherals
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();

    // FIXME ths stopped working??
    let mut pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();
    let _backup = pwrcfg.backup().unwrap();

    // Enable SRAM1-3
    dp.RCC.ahb2enr.modify(|_, w| {
        w.sram1en()
            .set_bit()
            .sram2en()
            .set_bit()
            .sram3en()
            .set_bit()
    });

    // TODO: RAMECC1, RAMECC2, RAMECC3

    pac::DWT::unlock();
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();
    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);

    // FIXME?
    // Ah, yes
    // Copy the PWR CR3 power register value from a working Arduino sketch and write the value
    // directly since I cannot for the life of me figure out how to get it working with the
    // provided power configuration methods.
    // This is obviously not the *proper way* to do things but it works. ~~The orange DL2 LED
    // is still lit meaning something is indeed not configured correctly.~~
    // PMIC is configured later, the DL2 LED issue was fixed by analyzing the Arduino Bootloader
    // I2C traffic.
    // core::ptr::write_volatile(0x5802480c as *mut u32, 0b00000101000000010000000001010110);

    // Constrain and Freeze clocks
    let ccdr = {
        let mut ccdr = dp
            .RCC
            .constrain()
            .bypass_hse()
            .sys_ck(480.MHz())
            .hclk(240.MHz())
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_q_ck(240.MHz())
            .pll2_strategy(rcc::PllConfigStrategy::Iterative)
            .pll2_p_ck(500.MHz() / 2)
            .pll2_q_ck(400.MHz() / 2)
            .pll2_r_ck(300.MHz() / 2)
            .pll3_strategy(rcc::PllConfigStrategy::Iterative)
            .pll3_p_ck(800.MHz() / 2)
            .pll3_q_ck(800.MHz() / 2)
            .pll3_r_ck(800.MHz() / 83)
            .freeze(pwrcfg, &dp.SYSCFG);

        // USB Clock
        let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
        ccdr.peripheral
            .kernel_usb_clk_mux(rcc::rec::UsbClkSel::Hsi48);
        ccdr.peripheral.kernel_adc_clk_mux(rcc::rec::AdcClkSel::Per);
        // ccdr.peripheral.kernel_usart234578_clk_mux(rcc::rec::Usart234578ClkSel::HsiKer);
        ccdr
    };

    // Make CRC available
    interrupt_free(|cs| {
        utils::CRC
            .borrow(cs)
            .replace(Some(dp.CRC.crc(ccdr.peripheral.CRC)))
    });

    // GPIO
    let (gpioa, gpiob, _gpioc, gpiod, gpioe, gpiof, gpiog, gpioh, gpioi, gpioj, gpiok) = {
        (
            dp.GPIOA.split(ccdr.peripheral.GPIOA),
            dp.GPIOB.split(ccdr.peripheral.GPIOB),
            dp.GPIOC.split(ccdr.peripheral.GPIOC),
            dp.GPIOD.split(ccdr.peripheral.GPIOD),
            dp.GPIOE.split(ccdr.peripheral.GPIOE),
            dp.GPIOF.split(ccdr.peripheral.GPIOF),
            dp.GPIOG.split(ccdr.peripheral.GPIOG),
            dp.GPIOH.split(ccdr.peripheral.GPIOH),
            dp.GPIOI.split(ccdr.peripheral.GPIOI),
            dp.GPIOJ.split(ccdr.peripheral.GPIOJ),
            dp.GPIOK.split(ccdr.peripheral.GPIOK),
        )
    };

    // Configure PK5, PK6, PK7 as output.
    let mut led_r = gpioi.pi12.into_push_pull_output();
    let mut led_g = gpioj.pj13.into_push_pull_output();
    let mut led_b = gpioe.pe3.into_push_pull_output();
    led_r.set_high();
    led_g.set_low();
    led_b.set_high();

    // FIXME
    // // Internal I2C bus
    // let mut internal_i2c = dp.I2C1.i2c(
    //     (
    //         gpiob.pb6.into_alternate::<4>().set_open_drain(), // scl
    //         gpiob.pb7.into_alternate::<4>().set_open_drain(), // sda
    //     ),
    //     100.kHz(),
    //     ccdr.peripheral.I2C1,
    //     &ccdr.clocks,
    // );
    // // Configure PMIC (NXP PF1550)
    // pmic::configure(&mut internal_i2c).unwrap();

    // // UART1 terminal
    // {
    //     let mut uart = dp
    //         .USART1
    //         .serial(
    //             (
    //                 gpioa.pa9.into_alternate::<7>(), // tx
    //                 gpiob.pb7.into_alternate::<7>(), // rx
    //             ),
    //             terminal::UART_TERMINAL_BAUD.bps(),
    //             ccdr.peripheral.USART1,
    //             &ccdr.clocks,
    //         )
    //         .unwrap();

    //     // UART interrupt
    //     uart.listen(hal::serial::Event::Rxne);
    //     cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);

    //     let (terminal_tx, terminal_rx) = uart.split();

    //     // FIXME it crashes here.
    //     interrupt_free(|cs| {
    //         terminal::UART_TERMINAL_TX
    //             .borrow(cs)
    //             .replace(Some(terminal_tx));
    //         terminal::UART_TERMINAL_RX
    //             .borrow(cs)
    //             .replace(Some(terminal_rx));
    //     });
    // };

    // gpiok.pk3.into_push_pull_output().set_high().unwrap(); // cable
    // gpiok.pk4.into_push_pull_output().set_low().unwrap(); // alt

    // // RTC
    // {
    //     // Configure RTC
    //     // FIXME - another crash
    //     TimeSource::set_source(rtc::Rtc::open_or_init(
    //         dp.RTC,
    //         backup.RTC,
    //         rtc::RtcClock::Lse {
    //             freq: 32768.Hz(),
    //             bypass: true,
    //             css: false,
    //         },
    //         &ccdr.clocks,
    //     ));
    //     // Set Date and Time
    //     #[cfg(debug_assertions)]
    //     let _ = TimeSource::set_date_time(
    //         NaiveDate::from_ymd_opt(
    //             consts::COMPILE_TIME_YEAR,
    //             consts::COMPILE_TIME_MONTH,
    //             consts::COMPILE_TIME_DAY,
    //         )
    //         .and_then(|td| {
    //             td.and_hms_opt(
    //                 consts::COMPILE_TIME_HOUR,
    //                 consts::COMPILE_TIME_MINUTE,
    //                 consts::COMPILE_TIME_SECOND,
    //             )
    //         })
    //         .unwrap(),
    //     );

    //     let now = TimeSource::get_date_time().unwrap();

    //     // Set boot time
    //     interrupt_free(|cs| time::BOOT_TIME.replace(cs, TimeSource::get_date_time()));
    // }

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // System temperature
    {
        // Temp ADC
        let mut channel = adc::Temperature::new();
        let mut temp_adc_disabled = adc::Adc::adc3(
            dp.ADC3,
            8.MHz(),
            &mut delay,
            ccdr.peripheral.ADC3,
            &ccdr.clocks,
        );
        temp_adc_disabled.set_sample_time(adc::AdcSampleTime::T_387);
        temp_adc_disabled.set_resolution(adc::Resolution::SixteenBit);
        temp_adc_disabled.calibrate();
        channel.enable(&temp_adc_disabled);
        delay.delay_us(25_u16); // Delay necessary?
        let temp_adc = temp_adc_disabled.enable();
        // Save clock freq
        interrupt_free(|cs| {
            system::CLOCK_FREQ
                .borrow(cs)
                .replace(Some(ccdr.clocks.sys_ck()));
            system::CORE_TEMP
                .borrow(cs)
                .replace(Some((temp_adc, channel)));
        });
    }

    // SDRAM
    let framebuffer_start_addr = {
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
        mem::sdram::configure(&cp.MPU, &cp.SCB);
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
        let framebuffer = sdram_ptr as usize;
        let heap = sdram_ptr as usize + display::FRAME_BUFFER_ALLOC_SIZE;
        log::info!("Framebuffer: {:p}", framebuffer as *const ());
        log::info!("Heap: {:p}", heap as *const ());
        mem::ALLOCATOR.init(heap, mem::HEAP_SIZE);

        framebuffer
    };

    // Enable osc
    {
        let mut oscen = gpioh.ph1.into_push_pull_output();
        delay.delay_ms(10u32);
        oscen.set_high();
        delay.delay_ms(10u32);
    }

    // SD Card
    {
        let sdcard = dp.SDMMC2.sdmmc(
            (
                gpiod.pd6.into_alternate::<11>().speed(Speed::VeryHigh),
                gpiod.pd7.into_alternate::<11>().speed(Speed::VeryHigh),
                gpiob.pb14.into_alternate::<9>().speed(Speed::VeryHigh),
                gpiob.pb15.into_alternate::<9>().speed(Speed::VeryHigh),
                gpiob.pb3.into_alternate::<9>().speed(Speed::VeryHigh),
                gpiob.pb4.into_alternate::<9>().speed(Speed::VeryHigh),
            ),
            ccdr.peripheral.SDMMC2,
            &ccdr.clocks,
        );
        interrupt_free(|cs| {
            fs::sdmmc_fs::SD_CARD
                .borrow(cs)
                .replace(Some(fs::sdmmc_fs::SdmmcFs::new(sdcard)))
        });
    }

    // QSPI Flash
    {
        let mut qspi_store = fs::qspi_store::NorFlash::new(
            dp.QUADSPI.bank1(
                (
                    gpiof.pf10.into_alternate::<9>().speed(Speed::VeryHigh),
                    gpiod.pd11.into_alternate::<9>().speed(Speed::VeryHigh),
                    gpiod.pd12.into_alternate::<9>().speed(Speed::VeryHigh),
                    gpiof.pf7.into_alternate::<9>().speed(Speed::VeryHigh),
                    gpiod.pd13.into_alternate::<9>().speed(Speed::VeryHigh),
                ),
                100.MHz(),
                &ccdr.clocks,
                ccdr.peripheral.QSPI,
            ),
            gpiog.pg6.into_push_pull_output().speed(Speed::VeryHigh),
        );
        qspi_store.init().unwrap();

        interrupt_free(|cs| {
            fs::qspi_store::QSPI_STORE
                .borrow(cs)
                .replace(Some(qspi_store));
        });
    }

    // FIXME
    // // Display config
    // {
    //     // let mut anx = Anx7625::new(
    //     //     gpiok.pk2.into_push_pull_output(),
    //     //     gpioj.pj3.into_push_pull_output(),
    //     //     gpioj.pj6.into_push_pull_output(),
    //     // );
    //     // anx.init(&mut internal_i2c, &mut delay).unwrap();
    //     // anx.wait_hpd_event(&mut internal_i2c, &mut delay); // Blocks until monitor is connected
    //     // log::info!("Monitor connected");
    //     // let edid = crate::utils::into_ok_or_err(
    //     //     anx.dp_get_edid(&mut internal_i2c, &mut delay)
    //     //         .map_err(|(_, edid)| unsafe { edid.assume_init() }),
    //     // );
    //     // log::warn!("{:#?}", edid);
    //     // anx.dp_start(&mut internal_i2c, &mut delay, &edid, anx7625::EdidModes::EDID_MODE_1024x768_60Hz)
    //     //     .unwrap();

    //     let mut ltdc = stm32h7xx_hal::ltdc::Ltdc::new(dp.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
    //     let display_config = DisplayConfiguration {
    //         active_width: display::SCREEN_WIDTH as u16,
    //         active_height: display::SCREEN_HEIGHT as u16,
    //         h_back_porch: 120,
    //         h_front_porch: 32,
    //         v_back_porch: 10,
    //         v_front_porch: 45,
    //         h_sync: 20,
    //         v_sync: 12,

    //         // horizontal synchronization: `false`: active low, `true`: active high
    //         h_sync_pol: false,
    //         // vertical synchronization: `false`: active low, `true`: active high
    //         v_sync_pol: false,
    //         // data enable: `false`: active low, `true`: active high
    //         not_data_enable_pol: true,
    //         // pixel_clock: `false`: active low, `true`: active high
    //         pixel_clock_pol: false,
    //     };
    //     ltdc.init(display_config);

    //     let fb0: &'static mut _ = &mut *(framebuffer_start_addr as *mut _);
    //     let fb1: &'static mut _ =
    //         &mut *((framebuffer_start_addr + display::FRAME_BUFFER_SIZE) as *mut _);
    //     let display = h7_display::H7Display::new(fb0, fb1);
    //     let gpu = display::Gpu::new(display, ltdc.split());

    //     interrupt_free(|cs| {
    //         display::GPU.borrow(cs).replace(Some(gpu));
    //     });

    //     // Set up frame sawp timer
    //     let mut timer2 =
    //         dp.TIM2
    //             .timer(display::FRAME_RATE.Hz(), ccdr.peripheral.TIM2, &ccdr.clocks);
    //     timer2.listen(hal::timer::Event::TimeOut);
    //     cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);

    //     let _dsihost = dsi::Dsi::new(
    //         dsi::DsiLanes::Two,
    //         &display_config,
    //         dp.DSIHOST,
    //         ccdr.peripheral.DSI,
    //         &ccdr.clocks,
    //     );
    // }

    let mut menu = terminal::menu::Menu::new(terminal::TerminalWriter, terminal::MENU);

    let mut cmd_buf = [0u8; 1024];
    let mut cmd_buf_len: usize = 0;

    // Main loop
    led_r.set_high();
    led_g.set_high();
    led_b.set_high();
    let _ = write!(menu.writer(), "> ");

    loop {
        match terminal::TERMINAL_INPUT_FIFO.dequeue() {
            Some(10) => match core::str::from_utf8(&cmd_buf[0..cmd_buf_len]) {
                Ok(s) => {
                    let mut parts = s.split_whitespace().filter(|l| !l.trim().is_empty());

                    if let Some(cmd) = parts.next() {
                        // Collect up to 16 arguments
                        let mut args = [""; 16];
                        let mut args_len = 0;
                        for arg in args.iter_mut() {
                            match parts.next() {
                                Some(s) => {
                                    *arg = s;
                                    args_len += 1;
                                }
                                None => break,
                            }
                        }
                        // Run command
                        if let Err(e) = menu.run(cmd, &args[0..args_len]) {
                            let _ = writeln!(menu.writer(), "Error: {e}");
                        }
                        // Clear input
                        delay.delay_ms(10u8); // Wait for interrupts
                        while terminal::TERMINAL_INPUT_FIFO.dequeue().is_some() {}
                        cmd_buf_len = 0;
                    }
                    let _ = write!(menu.writer(), "> ");
                }
                Err(e) => {
                    let _ = writeln!(menu, "Error: {e}");
                }
            },
            Some(c) => {
                if cmd_buf_len < cmd_buf.len() {
                    cmd_buf[cmd_buf_len] = c;
                    cmd_buf_len += 1;
                } else {
                    let _ = writeln!(menu, "Error: Buffer full");
                }
            }
            None => {} // FIFO empty
        };

        // Blink
        if let Some(dt) = TimeSource::get_date_time() {
            if dt.second() % 2 == 0 {
                led_b.set_high();
            } else {
                led_b.set_low();
            }
        }

        // FIXME -- Additional blink, as the above is disabled due to commenting out the RTC setup (since it is crashing at the moment).
        delay.delay_ms(500u32);
        led_b.set_high();
        delay.delay_ms(500u32);
        led_b.set_low();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("{:?}", layout)
}

#[cortex_m_rt::exception]
unsafe fn DefaultHandler(irqn: i16) -> ! {
    // https://www.keil.com/pack/doc/CMSIS/Core/html/group__NVIC__gr.html
    let name = match irqn {
        -14 => "NonMaskableInt_IRQn",
        -13 => "HardFault_IRQn",
        -12 => "MemoryManagement_IRQn",
        -11 => "BusFault_IRQn",
        -10 => "UsageFault_IRQn",
        -9 => "SecureFault_IRQn",
        -5 => "SVCall_IRQn",
        -4 => "DebugMonitor_IRQn",
        -2 => "PendSV_IRQn",
        -1 => "SysTick_IRQn",
        0 => "WWDG_STM_IRQn",
        1 => "PVD_STM_IRQn",
        _ => "<Unknown>",
    };

    panic!("IRQn: {} ({})", irqn, name);
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:?}", ef);
}
