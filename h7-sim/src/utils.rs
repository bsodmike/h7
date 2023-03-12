use std::time::{Duration, Instant};

#[macro_export]
macro_rules! sz_al_of {
    ($type:ty) => {
        println!(
            "{}: sz = {}, al = {}",
            stringify!($type),
            core::mem::size_of::<$type>(),
            core::mem::align_of::<$type>()
        )
    };
}

pub fn timer<D: core::fmt::Display, F>(text: D, mut func: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    func();
    let diff = Instant::now() - start;
    let fps = Duration::SECOND.as_micros() as f64 / diff.as_micros() as f64;
    println!("{text}: {diff:?}, FPS: {fps:.02}");
}
