# h7

## Todo

### Sim

* [ ] Create display crate `h7-display` (lib)
* [ ] Create sim crate `h7-sim` (lib, bin)
* [ ] Allow the `h7-sim` crate to be used a library and a binary.
    - As lib: When used as a library, provide sim entry point
    - As bin: Show display, open h7-app and run .so

### Hardware/HAL/Driver

* [x] External SDRAM
* [x] Allocator using external SDRAM
* [ ] Displayport output
* [ ] USB Keyboard
* [x] ~~USB Serial communication~~
* [ ] Microchip/NXP Crypto chip. NXP SE050C2 does not have a rust driver. Microchip [ATECC608A](https://crates.io/crates/Rusty_CryptoAuthLib) does.
* [x] SDMMC
* [ ] Multi-Core. Move IO to CM4 and reserve CM7 for applications.
* [ ] Ethernet
* [ ] WiFi
* [ ] Bluetooth

### Software

* [ ] Interrupt prio
* [x] Add release/debug info to osinfo.
* [ ] CPU Temp ADC interrupt.
* [ ] Watchdog info in mcuinfo.
* [ ] Watchdog control command.
* [x] RTC control command. `date set [date time|date|time]`
* [ ] Render to display. Interrupt driven frame-updates.
* [ ] USB input with interrupts.
* [ ] User login using secure element.
* [x] Shell.
* [x] Application API. (wip)
* [x] Load binaries from SD Card [~~(async?)~~](https://github.com/stm32-rs/stm32h7xx-hal/issues/227)
* [x] CRC with verification
* [x] Run programs without crashing (duh)
* [ ] Settings storage? NOR-Flash/SD Card?
* [ ] Settings using hds::Kv
* [ ] Show long names on SD Card
* [ ] HardFault info (upstream to cortex_m?)


### Test

* [x] Test SDRAM https://github.com/stm32-rs/stm32f7xx-hal/blob/master/examples/fmc.rs#L110

