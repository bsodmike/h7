use embedded_hal::blocking::i2c::Write as I2CWrite;

#[allow(dead_code)]
const PMIC_ADDRESS: u8 = 0x08;

#[allow(dead_code)]
const PMIC_SETUP: &[&[u8]] = &[
    &[0x4F, 0x00],
    &[0x50, 0x0F],
    &[0x4C, 0x05],
    &[0x4D, 0x03],
    &[0x52, 0x09],
    &[0x53, 0x0F],
    &[0x9C, 0x80],
    &[0x9E, 0x20],
    &[0x42, 0x02],
    &[0x94, 0xA0],
    &[0x3B, 0x0F],
    &[0x35, 0x0F],
    &[0x42, 0x01],
];

#[allow(dead_code)]
pub fn configure<E, BUS: I2CWrite<Error = E>>(bus: &mut BUS) -> Result<(), E> {
    for data in PMIC_SETUP {
        bus.write(PMIC_ADDRESS, data)?;
    }
    Ok(())
}
