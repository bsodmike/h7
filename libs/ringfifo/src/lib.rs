#![no_std]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]

// TODO:
// * Make separate library
// * Implement: iterator stuff, make contiguous

use core::mem::MaybeUninit;

#[derive(Debug)]
pub struct RingFiFo<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    size: usize,
    r: usize,
    w: usize,
}

impl<T, const N: usize> RingFiFo<T, N> {
    pub const fn new() -> Self {
        Self {
            buf: MaybeUninit::uninit_array::<N>(),
            size: 0,
            r: 0,
            w: 0,
        }
    }

    pub fn push_back(&mut self, item: T) {
        if self.is_full() {
            self.r = self.next_r();
        } else {
            self.inc_size();
        }
        self.buf[self.w].write(item);
        self.w = self.next_w();
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.dec_size();
            let v = core::mem::replace(&mut self.buf[self.r], MaybeUninit::uninit());
            self.r = self.next_r();
            Some(unsafe { v.assume_init() })
        }
    }

    pub const fn is_full(&self) -> bool {
        self.size == N
    }

    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    const fn next_w(&self) -> usize {
        (self.w + 1) % N
    }

    const fn next_r(&self) -> usize {
        (self.r + 1) % N
    }

    fn inc_size(&mut self) {
        if self.size < N {
            self.size += 1;
        }
    }

    fn dec_size(&mut self) {
        if self.size > 0 {
            self.size -= 1;
        }
    }

    // fn previous_w(&self) -> usize {
    //     if self.w as isize - 1 < 0 {
    //         N - 1
    //     } else {
    //         self.w - 1
    //     }
    // }

    // fn previous_r(&self) -> usize {
    //     if self.r as isize - 1 < 0 {
    //         N - 1
    //     } else {
    //         self.r - 1
    //     }
    // }
}
