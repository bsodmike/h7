#![no_std]
#![feature(generic_const_exprs)]
// #![feature(maybe_uninit_uninit_array)]
// #![feature(const_maybe_uninit_uninit_array)]
#![allow(mutable_transmutes)]
#![feature(const_mut_refs)]

mod display;
mod framebuffer;

pub use {display::H7Display, framebuffer::FrameBuffer};
