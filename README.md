# h7

## Todo

### Hardware/HAL

* [x] External SDRAM
* [x] Allocator using external SDRAM
* [ ] Displayport output
* [ ] USB Keyboard
* [ ] Microchip/NXP Crypto chip. NXP SE050C2 does not have a rust driver. Microchip [ATECC608A](https://crates.io/crates/Rusty_CryptoAuthLib) does.
* [x] SDMMC
* [ ] Multi-Core. Move IO to CM4 and reserve CM7 for applications.
* [ ] Ethernet
* [ ] WiFi
* [ ] Bluetooth

### Software

* [ ] CPU Temp ADC interrupt.
* [ ] Watchdog info in mcuinfo.
* [ ] Watchdog control command.
* [ ] Render to display. Interrupt driven frame-updates.
* [ ] USB input with interrupts.
* [ ] User login using secure element.
* [x] Shell.
* [ ] Application API.
* [ ] Load binaries from SD Card [~~(async?)~~](https://github.com/stm32-rs/stm32h7xx-hal/issues/227)
* [ ] Settings storage? NOR-Flash/SD Card?

### Test

* [ ] Test SDRAM https://github.com/stm32-rs/stm32f7xx-hal/blob/master/examples/fmc.rs#L110

