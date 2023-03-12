use {core::mem::MaybeUninit, embedded_graphics_core::prelude::*};

#[derive(Debug)]
#[repr(transparent)]
pub struct FrameBuffer<COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]:,
{
    buffer: [MaybeUninit<COLOR>; WIDTH * HEIGHT],
}

impl<COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize> FrameBuffer<COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    // pub const fn new() -> Self {
    //     Self {
    //         buffer: MaybeUninit::<COLOR>::uninit_array(),
    //     }
    // }

    #[inline(always)]
    pub fn at(&self, x: usize, y: usize) -> &COLOR {
        let idx = Self::xy_to_index(x, y);
        // &self[idx]
        unsafe { self.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut COLOR {
        let idx = Self::xy_to_index(x, y);
        // &mut self[idx]
        unsafe { self.get_unchecked_mut(idx) }
    }

    #[inline(always)]
    pub fn xy_to_index(x: usize, y: usize) -> usize {
        x + (WIDTH * y)
    }

    // #[inline(always)]
    // pub unsafe fn at_unchecked(&self, x: usize, y: usize) -> &COLOR {
    //     let idx = self.xy_to_index(x, y);
    //     self.get_unchecked(idx)
    // }

    // #[inline(always)]
    // pub unsafe fn at_unchecked_mut(&mut self, x: usize, y: usize) -> &mut COLOR {
    //     let idx = self.xy_to_index(x, y);
    //     self.get_unchecked_mut(idx)
    // }
}

impl<COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize> core::ops::Deref
    for FrameBuffer<COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    type Target = [COLOR; WIDTH * HEIGHT];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(&self.buffer) }
    }
}

impl<COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize> core::ops::DerefMut
    for FrameBuffer<COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(&mut self.buffer) }
    }
}
