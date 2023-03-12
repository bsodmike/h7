pub mod qspi_store;
pub mod sdmmc_fs;
pub mod sdram;

use cortex_m_alloc::CortexMHeap;

// Heap allocator
#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub const HEAP_SIZE: usize = sdram::SDRAM_SIZE - crate::display::FRAME_BUFFER_ALLOC_SIZE;
