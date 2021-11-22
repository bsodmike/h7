// USB
pub static mut USB_MEMORY_1: [u32; 1024] = [0u32; 1024];

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

use core::convert::TryFrom;
use usb_device::class_prelude::*;
use usb_device::control::RequestType;

const GET_OS_FEATURE: u8 = b'A';

#[allow(non_snake_case)]
#[repr(u16)]
pub enum OSFeatureDescriptorType {
    CompatibleID = 4,
    Properties = 5,
}

impl TryFrom<u16> for OSFeatureDescriptorType {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(Self::CompatibleID),
            5 => Ok(Self::Properties),
            _ => Err(()),
        }
    }
}

const MS_COMPATIBLE_ID_DESCRIPTOR: [u8; 40] = [
    0x28, 0x00, 0x00, 0x00, // Length 40 bytes
    0x00, 0x01, // Version
    0x04, 0x00, // Compatibility ID Descriptor index
    0x01, // Number of sections
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved: 7 bytes
    0x01, // Interface Number
    0x01, // Reserved
    b'W', b'I', b'N', b'U', b'S', b'B', 0x00, 0x00, // Compatible ID: 8 bytes ASCII
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Sub-Compatible ID: 8 bytes ASCII
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved: 6 bytes
];

const IF2_MS_PROPERTIES_OS_DESCRIPTOR: [u8; 142] = [
    0x8e, 0x00, 0x00, 0x00, // Length, 40 bytes
    0x00, 0x01, // Version
    0x05, 0x00, // wIndex: properties
    0x01, 0x00, // wCount: a single property
    0x84, 0x00, 0x00, 0x00, // Property length
    0x01, 0x00, 0x00, 0x00, // dwPropertyDataType: REG_SZ
    0x28, 0x00, // name length
    b'D', 0, b'e', 0, b'v', 0, b'i', 0, b'c', 0, b'e', 0, b'I', 0, b'n', 0, b't', 0, b'e', 0, b'r',
    0, b'f', 0, b'a', 0, b'c', 0, b'e', 0, b'G', 0, b'U', 0, b'I', 0, b'D', 0, 0, 0, 0x4e, 0x00,
    0x00, 0x00, // data length
    b'{', 0, b'C', 0, b'D', 0, b'B', 0, b'3', 0, b'B', 0, b'5', 0, b'A', 0, b'D', 0, b'-', 0, b'2',
    0, b'9', 0, b'3', 0, b'B', 0, b'-', 0, b'4', 0, b'6', 0, b'6', 0, b'3', 0, b'-', 0, b'A', 0,
    b'A', 0, b'3', 0, b'6', 0, b'-', 0, b'1', 0, b'A', 0, b'A', 0, b'E', 0, b'4', 0, b'6', 0, b'4',
    0, b'6', 0, b'3', 0, b'7', 0, b'7', 0, b'6', 0, b'}', 0, 0, 0,
];

pub struct MicrosoftDescriptors;

impl<B: UsbBus> UsbClass<B> for MicrosoftDescriptors {
    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();
        if req.request_type != RequestType::Vendor {
            return;
        }

        if req.request == GET_OS_FEATURE {
            match OSFeatureDescriptorType::try_from(req.index) {
                Ok(OSFeatureDescriptorType::CompatibleID) => {
                    // Handle request for an Extended Compatible ID Descriptor.
                    // Interface number is ignored as there is only one device-wide
                    // Compatible ID Descriptor.
                    xfer.accept_with_static(&MS_COMPATIBLE_ID_DESCRIPTOR).ok();
                }
                Ok(OSFeatureDescriptorType::Properties) => {
                    // Handle request for an Extended Properties OS Descriptor.
                    match req.value as u8 {
                        2 => {
                            xfer.accept_with_static(&IF2_MS_PROPERTIES_OS_DESCRIPTOR)
                                .ok();
                        }
                        _ => {
                            xfer.reject().ok();
                        }
                    }
                }
                _ => {
                    xfer.reject().ok();
                }
            }
        }
    }
}
