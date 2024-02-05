use crate::Led;
use core::{cell::RefCell, mem};
use critical_section::Mutex;
use embedded_display_controller::{DisplayControllerLayer, PixelFormat};
use h7_display::{FrameBuffer, H7Display};
use stm32h7xx_hal::{interrupt, ltdc::LtdcLayer1};

type Pixel = embedded_graphics::pixelcolor::Rgb565;

pub const SCREEN_WIDTH: usize = 480;
pub const SCREEN_HEIGHT: usize = 800;
pub const FRAME_BUFFER_SIZE: usize =
    mem::size_of::<FrameBuffer<Pixel, SCREEN_WIDTH, SCREEN_HEIGHT>>();
pub const FRAME_BUFFER_ALLOC_SIZE: usize = FRAME_BUFFER_SIZE * 2;
pub const FRAME_RATE: u32 = 60;

pub static GPU: Mutex<RefCell<Option<Gpu>>> = Mutex::new(RefCell::new(None));

pub struct Gpu {
    display: H7Display<'static, Pixel, SCREEN_HEIGHT, SCREEN_HEIGHT>,
    layer: LtdcLayer1,
}

impl Gpu {
    pub fn new(
        display: H7Display<'static, Pixel, SCREEN_HEIGHT, SCREEN_HEIGHT>,
        mut layer: LtdcLayer1,
    ) -> Self {
        unsafe {
            layer.enable(
                display.front_buffer().as_ptr() as *const u16,
                PixelFormat::RGB565,
            )
        };
        Self { display, layer }
    }

    pub fn swap(&mut self) {
        if !self.layer.is_swap_pending() {
            let (front, _) = self.display.swap_buffers();
            unsafe { self.layer.swap_framebuffer(front.as_ptr() as *const u16) };
            unsafe { Led::Blue.toggle() };
        }
        unsafe { Led::Red.toggle() };

        // while self.layer.is_swap_pending() {
        //     cortex_m::asm::nop();
        // }
    }
}

impl core::ops::Deref for Gpu {
    type Target = H7Display<'static, Pixel, SCREEN_HEIGHT, SCREEN_HEIGHT>;

    fn deref(&self) -> &Self::Target {
        &self.display
    }
}

impl core::ops::DerefMut for Gpu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.display
    }
}

// Interrupt to swap framebuffers
#[interrupt]
fn TIM2() {
    unsafe {
        crate::utils::interrupt_free(|cs| {
            GPU.borrow(cs).borrow_mut().as_mut().unwrap().swap();
        });
        stm32h7xx_hal::pac::TIM2::ptr()
            .as_ref()
            .unwrap()
            .sr
            .write(|w| w.uif().clear_bit());
    };
}

#[interrupt]
fn LTDC() {
    unsafe { Led::Red.on() };
}

#[interrupt]
fn DMA2D() {
    unsafe { Led::Red.on() };
}

#[interrupt]
fn DSI() {
    unsafe { Led::Red.on() };
}
