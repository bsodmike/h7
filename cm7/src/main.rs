#![no_main]
#![no_std]
#![feature(alloc_error_handler, result_into_ok_or_err)]

extern crate alloc;

use {
    // anx7625::Anx7625,
    chrono::{NaiveDate, Timelike},
    core::fmt::Write,
    cortex_m::interrupt::free as interrupt_free,
    cortex_m_alloc::CortexMHeap,
    // embedded_display_controller::{
    //     DisplayConfiguration, DisplayController, DisplayControllerLayer, PixelFormat,
    // },
    led::LED,
    stm32h7xx_hal::{
        self as hal, adc,
        hal::digital::v2::OutputPin,
        pac,
        prelude::*,
        rcc, rtc,
        usb_hs::{UsbBus, USB1_ULPI},
    },
    time::TimeSource,
    usb_device::prelude::*,
};

mod consts;
mod led;
#[cfg(feature = "semihosting")]
mod logger;
#[cfg(not(feature = "semihosting"))]
mod panic;
mod pmic;
mod sdmmc_fs;
mod sdram;
mod system;
mod terminal;
mod time;
mod usb;

// Heap allocator
#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    #[cfg(feature = "semihosting")]
    logger::init();

    // Get peripherals
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    pac::DWT::unlock();
    cp.DWT.enable_cycle_counter();
    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);

    // Ah, yes
    // Copy the PWR CR3 power register value from a working Arduino sketch and write the value
    // directly since I cannot for the life of me figure out how to get it working with the
    // provided power configuration methods.
    // This is obviously not the *proper way* to do things but it works. ~~The orange DL2 LED
    // is still lit meaning something is indeed not configured correctly.~~
    // PMIC is configured later, the DL2 LED issue was fixed by analyzing the Arduino Bootloader
    // I2C traffic.
    core::ptr::write_volatile(0x5802480c as *mut u32, 0b00000101000000010000000001010110);

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let mut pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();
    let backup = pwrcfg.backup().unwrap();

    // Constrain and Freeze clocks
    let ccdr = {
        let mut ccdr = dp
            .RCC
            .constrain()
            .bypass_hse()
            .sys_ck(480.mhz())
            .hclk(240.mhz())
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_q_ck(240.mhz())
            .pll2_strategy(rcc::PllConfigStrategy::Iterative)
            .pll2_p_ck(100.mhz())
            .pll3_strategy(rcc::PllConfigStrategy::Iterative)
            .pll3_p_ck(100.mhz())
            .pll3_r_ck(100.mhz())
            .freeze(pwrcfg, &dp.SYSCFG);

        // USB Clock
        let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
        ccdr.peripheral
            .kernel_usb_clk_mux(rcc::rec::UsbClkSel::HSI48);
        ccdr
    };

    // GPIO
    let (gpioa, gpiob, gpioc, gpiod, gpioe, gpiof, gpiog, gpioh, gpioi, gpioj, gpiok) = {
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
    let mut led_r = gpiok.pk5.into_push_pull_output();
    let mut led_g = gpiok.pk6.into_push_pull_output();
    let mut led_b = gpiok.pk7.into_push_pull_output();
    led_r.set_high().unwrap();
    led_g.set_high().unwrap();
    led_b.set_low().unwrap();

    // gpiok.pk3.into_push_pull_output().set_high().unwrap(); // cable
    // gpiok.pk4.into_push_pull_output().set_low().unwrap(); // alt

    // RTC
    {
        // Configure RTC
        TimeSource::set_source(rtc::Rtc::open_or_init(
            dp.RTC,
            backup.RTC,
            rtc::RtcClock::Lse {
                freq: 32768.hz(),
                bypass: true,
                css: false,
            },
            &ccdr.clocks,
        ));
        // Set Date and Time
        #[cfg(debug_assertions)]
        let _ = TimeSource::set_date_time(
            NaiveDate::from_ymd(
                consts::COMPILE_TIME_YEAR,
                consts::COMPILE_TIME_MONTH,
                consts::COMPILE_TIME_DAY,
            )
            .and_hms(
                consts::COMPILE_TIME_HOUR,
                consts::COMPILE_TIME_MINUTE,
                consts::COMPILE_TIME_SECOND,
            ),
        );
    }

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // System temperature
    {
        // Temp ADC
        let mut channel = adc::Temperature::new();
        let mut temp_adc_disabled =
            adc::Adc::adc3(dp.ADC3, &mut delay, ccdr.peripheral.ADC3, &ccdr.clocks);
        temp_adc_disabled.set_sample_time(adc::AdcSampleTime::T_387);
        temp_adc_disabled.set_resolution(adc::Resolution::SIXTEENBIT);
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

    // Internal I2C bus
    let mut internal_i2c = dp.I2C1.i2c(
        (
            gpiob.pb6.into_alternate_af4().set_open_drain(),
            gpiob.pb7.into_alternate_af4().set_open_drain(),
        ),
        100.khz(),
        ccdr.peripheral.I2C1,
        &ccdr.clocks,
    );
    // Configure PMIC (NXP PF1550)
    pmic::configure(&mut internal_i2c).unwrap();

    // SDRAM
    {
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
    }

    // Enable osc
    {
        let mut oscen = gpioh.ph1.into_push_pull_output();
        delay.delay_ms(10u32);
        oscen.set_high().unwrap();
        delay.delay_ms(1000u32);
    }

    // SD Card
    {
        let sdcard = dp.SDMMC2.sdmmc(
            (
                gpiod.pd6.into_alternate_af11(),
                gpiod.pd7.into_alternate_af11(),
                gpiob.pb14.into_alternate_af9(),
                gpiob.pb15.into_alternate_af9(),
                gpiob.pb3.into_alternate_af9(),
                gpiob.pb4.into_alternate_af9(),
            ),
            ccdr.peripheral.SDMMC2,
            &ccdr.clocks,
        );
        interrupt_free(|cs| {
            sdmmc_fs::SD_CARD
                .borrow(cs)
                .replace(Some(sdmmc_fs::SdmmcFs::new(sdcard)))
        });
    }

    // Display config
    // {
    //     let mut anx = Anx7625::new(
    //         gpiok.pk2.into_push_pull_output(),
    //         gpioj.pj3.into_push_pull_output(),
    //         gpioj.pj6.into_push_pull_output(),
    //     );
    //     anx.init(&mut internal_i2c, &mut delay).unwrap();
    //     anx.wait_hpd_event(&mut internal_i2c, &mut delay); // Blocks until monitor is connected
    //     log::info!("Monitor connected");
    //     let edid = anx
    //         .dp_get_edid(&mut internal_i2c, &mut delay)
    //         .map_err(|(_, edid)| unsafe { edid.assume_init() })
    //         .into_ok_or_err();
    //     log::warn!("{:#?}", edid);
    //     anx.dp_start(
    //         &mut internal_i2c,
    //         &mut delay,
    //         &edid,
    //         anx7625::EdidModes::EDID_MODE_640x480_60Hz,
    //     )
    //     .unwrap();
    // }

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
    // let framebuf = alloc::vec::Vec::<u8>::with_capacity(1280 * 768);
    // let framebuf = alloc::vec![0xf81fu16; 640 * 480 * 2];
    // layer1.enable(framebuf.as_ptr(), PixelFormat::L8);
    // layer1.swap_framebuffer(framebuf.as_ptr());

    // UART1 terminal
    {
        let mut uart = dp
            .USART1
            .serial(
                (
                    gpioa.pa9.into_alternate_af7(),
                    gpioa.pa10.into_alternate_af7(),
                ),
                terminal::UART_TERMINAL_BAUD.bps(),
                ccdr.peripheral.USART1,
                &ccdr.clocks,
            )
            .unwrap();

        // UART interrupt
        uart.listen(hal::serial::Event::Rxne);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);

        let (terminal_tx, terminal_rx) = uart.split();
        interrupt_free(|cs| {
            terminal::UART_TERMINAL_TX
                .borrow(cs)
                .replace(Some(terminal_tx));
            terminal::UART_TERMINAL_RX
                .borrow(cs)
                .replace(Some(terminal_rx));
        });
    };

    // USB Serial
    {
        // Set OTG pin floating
        let mut _usb_otg = gpioj.pj6.into_floating_input();

        // Reset USB Phy
        let mut usb_phy_rst = gpioj.pj4.into_push_pull_output();
        usb_phy_rst.set_low().unwrap();
        delay.delay_ms(10u8);
        usb_phy_rst.set_high().unwrap();
        delay.delay_ms(10u8);

        // Enable USB OTG_HS interrupt
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::OTG_HS);

        let usb = USB1_ULPI::new(
            dp.OTG1_HS_GLOBAL,
            dp.OTG1_HS_DEVICE,
            dp.OTG1_HS_PWRCLK,
            gpioa.pa5.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpioi.pi11.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpioh.ph4.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpioc.pc0.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpioa.pa3.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb0.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb1.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb10.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb11.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb12.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb13.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            gpiob.pb5.into_alternate_af10(), //.set_speed(Speed::VeryHigh),
            ccdr.peripheral.USB1OTG,
            &ccdr.clocks,
        );

        usb::USB_BUS_ALLOCATOR = Some(UsbBus::new(usb, &mut usb::USB_MEMORY_1));

        let usb_serial = usbd_serial::SerialPort::new(usb::USB_BUS_ALLOCATOR.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(
            usb::USB_BUS_ALLOCATOR.as_ref().unwrap(),
            UsbVidPid(usb::VID, usb::PID),
        )
        .manufacturer("Arduino")
        .product("H7 Embedded Computer")
        .serial_number("dev-01")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .device_sub_class(2)
        .self_powered(false)
        .max_power(500)
        .max_packet_size_0(64)
        .build();

        interrupt_free(|cs| {
            usb::USB_DEVICE.borrow(cs).replace(Some(usb_dev));
            usb::SERIAL_PORT.borrow(cs).replace(Some(usb_serial));
        });
    }

    let mut menu = menu::Menu::new(terminal::TerminalWriter, terminal::MENU);

    let mut cmd_buf = [0u8; 64];
    let mut cmd_buf_len: usize = 0;

    // Main loop
    led_b.set_high().unwrap();
    let _ = write!(menu.writer(), "> ");

    loop {
        match interrupt_free(|cs| {
            terminal::TERMINAL_INPUT_FIFO
                .borrow(cs)
                .borrow_mut()
                .pop_front()
        }) {
            Some(10) => match core::str::from_utf8(&cmd_buf[0..cmd_buf_len]) {
                Ok(s) => {
                    let mut parts = s.trim().split_whitespace().filter(|l| !l.trim().is_empty());

                    if let Some(cmd) = parts.next() {
                        // Collect up to 8 arguments
                        let mut args = [""; 8];
                        let mut args_len = 0;
                        for i in 0..args.len() {
                            match parts.next() {
                                Some(s) => {
                                    args[i] = s;
                                    args_len += 1;
                                }
                                None => break,
                            }
                        }
                        // Run command
                        if let Err(e) = menu.run(cmd, &args[0..args_len]) {
                            let _ = writeln!(menu, "Error: {}", e);
                        }
                        cmd_buf_len = 0;
                    }
                    let _ = write!(menu.writer(), "> ");
                }
                Err(e) => {
                    let _ = writeln!(menu, "Error: {}", e);
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
            led_r.set_high().unwrap();
            if dt.second() % 2 == 0 {
                led_g.set_high().unwrap();
            } else {
                led_g.set_low().unwrap();
            }
        }
    }
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
