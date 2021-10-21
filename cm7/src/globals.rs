use {
    core::cell::RefCell, cortex_m::interrupt::Mutex, cortex_m_alloc::CortexMHeap,
    stm32h7xx_hal::rtc,
};

// Heap allocator
#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Time
pub static RTC: Mutex<RefCell<Option<rtc::Rtc>>> = Mutex::new(RefCell::new(None));

// USB
pub static mut USB_MEMORY_1: [u32; 1024] = [0u32; 1024];
