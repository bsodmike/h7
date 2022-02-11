#![no_std]

pub type AppEntryPoint = extern "C" fn(*const H7Api) -> i32;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct H7Api {
    // Sys, Mem
    pub alloc: extern "C" fn(size: usize, align: usize) -> *mut u8,
    pub free: extern "C" fn(ptr: *mut u8),
    pub panic: extern "C" fn(start: *const u8, len: usize) -> !,
    // IO
    pub getc: extern "C" fn() -> u8,
    pub putc: extern "C" fn(c: u8) -> i32,
    pub puts: extern "C" fn(start: *const u8, len: usize) -> i32,
    // GPU
    // pub screen_width_px: extern "C" fn() -> u32,
    // pub screen_height_px: extern "C" fn() -> u32,
    // pub screen_width_char: extern "C" fn() -> u32,
    // pub screen_height_char: extern "C" fn() -> u32,
    // pub dot: extern "C" fn(x1: u32, y1: u32, color: u16) -> i32,
    // pub line: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, stroke: u32, color: u16) -> i32,
    // pub square: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, stroke: u32, color: u16) -> i32,
    // pub square_fill: extern "C" fn(x1: u32, y1: u32, x2: u32, y2: u32, color: u16) -> i32,
}
