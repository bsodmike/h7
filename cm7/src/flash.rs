// let mut flash_cs = gpiog.pg6.into_push_pull_output();
// flash_cs.set_high().unwrap();
// let mut flash_qspi = dp.QUADSPI.bank1(
//     (
//         gpiof.pf10.into_alternate_af9(),
//         gpiod.pd11.into_alternate_af9(),
//         gpiod.pd12.into_alternate_af9(),
//         gpiof.pf7.into_alternate_af9(),
//         gpiod.pd13.into_alternate_af9(),
//     ),
//     133.mhz(),
//     &ccdr.clocks,
//     ccdr.peripheral.QSPI,
// );
