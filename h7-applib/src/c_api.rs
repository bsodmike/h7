use crate::Host;

#[no_mangle]
pub unsafe extern "C" fn h7_puts(s: *const u8) -> i32 {
    let slice = core::slice::from_raw_parts(s, util::strlen(s));
    match core::str::from_utf8(slice) {
        Ok(s) => Host::puts(s),
        Err(_) => -1,
    }
}

mod util {
    pub(crate) unsafe fn strlen(mut s: *const u8) -> usize {
        let mut result = 0;
        while *s != 0 {
            s = s.offset(1);
            result += 1;
        }
        result
    }
}
