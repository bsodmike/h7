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

// cp.NVIC
//     .set_priority(stm32h7xx_hal::pac::interrupt::OTG_HS, 1);
// cortex_m::peripheral::NVIC::unmask(stm32h7xx_hal::pac::interrupt::OTG_HS);
