use {cortex_m_alloc::CortexMHeap, stm32h7xx_hal::rtc};

// Heap allocator
#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Time
pub static mut RTC: Option<rtc::Rtc> = None;
