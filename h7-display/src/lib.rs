// #![no_std]
#![cfg_attr(target_os = "none", no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
// #![feature(maybe_uninit_uninit_array)]
// #![feature(const_maybe_uninit_uninit_array)]
#![allow(mutable_transmutes)]
#![feature(const_mut_refs)]
#![allow(dead_code)]

mod display;
mod framebuffer;

pub use {display::H7Display, framebuffer::FrameBuffer};
