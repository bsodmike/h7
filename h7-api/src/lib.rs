#![no_std]

#[derive(Debug, Clone)]
#[repr(C)]
pub struct H7Api {
    // pub gpu: H7Gpu,
    // pub getc: extern "C" fn() -> u8,
    // pub putc: extern "C" fn(c: u8) -> i32,
    pub puts: extern "C" fn(start: *const u8, len: usize) -> i32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct H7Gpu {
    pub dot: extern "C" fn(x1: u32, y1: u32, color: u16) -> i32,
    pub line: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, stroke: u32, color: u16) -> i32,
    pub square: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, stroke: u32, color: u16) -> i32,
    pub square_fill: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, color: u16) -> i32,
}
