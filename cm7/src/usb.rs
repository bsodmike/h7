use {
    crate::terminal,
    core::cell::RefCell,
    cortex_m::interrupt::{free as interrupt_free, Mutex},
    stm32h7xx_hal::{interrupt, usb_hs::USB1_ULPI},
    synopsys_usb_otg::UsbBus,
    usb_device::{bus::UsbBusAllocator, device::UsbDevice},
    usbd_serial::{DefaultBufferStore, SerialPort},
};

pub const VID: u16 = 0x2341;
pub const PID: u16 = 0x025b;

pub static mut USB_MEMORY_1: [u32; 1024] = [0u32; 1024];
pub static mut USB_BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus<USB1_ULPI>>> = None;
pub static SERIAL_PORT: Mutex<
    RefCell<Option<SerialPort<UsbBus<USB1_ULPI>, DefaultBufferStore, DefaultBufferStore>>>,
> = Mutex::new(RefCell::new(None));
pub static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<UsbBus<USB1_ULPI>>>>> =
    Mutex::new(RefCell::new(None));

pub struct SerialWriter<'s>(
    pub &'s mut SerialPort<'static, UsbBus<USB1_ULPI>, DefaultBufferStore, DefaultBufferStore>,
);

impl<'s> core::fmt::Write for SerialWriter<'s> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0
            // TODO: Loop until s.len() matched bytes written
            .write(s.as_bytes())
            .map(|_| ())
            .map_err(|_| core::fmt::Error)
    }
}

#[interrupt]
fn OTG_HS() {
    interrupt_free(|cs| {
        if let (Some(port), Some(device)) = (
            SERIAL_PORT.borrow(cs).borrow_mut().as_mut(),
            USB_DEVICE.borrow(cs).borrow_mut().as_mut(),
        ) {
            if device.poll(&mut [port]) {
                let mut buf = [0u8; 64];
                if let Ok(len) = port.read(&mut buf) {
                    let mut fifo = terminal::TERMINAL_INPUT_FIFO.borrow(cs).borrow_mut();
                    for b in &buf[0..len] {
                        let _ = fifo.push(*b);
                    }
                }
            }
        }
    })
}
