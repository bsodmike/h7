use std::{env, fs};

const ARM_ADDR_ALIGN: u32 = 4;
const THUMB_ADDR_ALIGN: u32 = 2;
const THUMB_MASK: u32 = 0x0000_0001;

fn main() {
    let mut args = env::args().skip(1);
    let input = args.next().expect("No input");
    let output = args.next().expect("No output");

    println!("input = {}", input);
    println!("output = {}", output);

    let input_data = fs::read(&input).unwrap();
    if input_data.len() < 4 {
        panic!("Not ennough data");
    }

    let boot_address =
        u32::from_le_bytes([input_data[0], input_data[1], input_data[2], input_data[3]]);
    println!("Boot address: 0x{:08x} ({})", boot_address, {
        // LSB is not part of the actual address,
        // but rather indicate if the cpu should
        // switch to arm or thumb mode.
        // 0 = ARM, 1 = THUMB
        let a = boot_address & !THUMB_MASK;
        match (
            boot_address & THUMB_MASK, // Thumb?
            a % THUMB_ADDR_ALIGN,      // Valid Thumb alignment?
            a % ARM_ADDR_ALIGN,        // Valid ARM alignment?
        ) {
            (1, 0, _) => "valid thumb",
            (0, _, 0) => "valid arm",
            _ => "invalid",
        }
    });

    let mut output_data = vec![0u8; input_data.len() + 4];
    // Convert address to big endian (the proper way...)
    output_data[..4].copy_from_slice(&boot_address.to_be_bytes());
    output_data[4..input_data.len()].copy_from_slice(&input_data[4..]);
    let crc = crc::Crc::<u32>::new(&crc::CRC_32_MPEG_2);
    let mut digest = crc.digest();
    digest.update(&output_data[..input_data.len()]);
    let crc_value = digest.finalize();
    println!("CRC: 0x{:08x}", crc_value);
    // Store CRC as big endian as well
    output_data[input_data.len()..].copy_from_slice(&crc_value.to_be_bytes());

    fs::write(&output, &output_data).unwrap();
    println!("Size: {} bytes", output_data.len());
    println!("Done");
}
