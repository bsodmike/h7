use {
    crate::{
        mem,
        terminal::{TerminalWriter, TERMINAL_INPUT_FIFO},
        utils,
    },
    core::{alloc::GlobalAlloc, cell::RefCell, fmt::Write},
    cortex_m::interrupt::Mutex,
    h7_api::{AppEntryPoint, H7Api},
};

const ARM_ADDR_ALIGN: usize = 4;
const THUMB_ADDR_ALIGN: usize = 2;
const THUMB_MASK: usize = 0x0000_0001;

pub const APP_START: *mut u8 = 0x2400_0000usize as *mut u8;
pub const APP_SIZE: usize = 512 * 1024;

// pub const APP_START: *mut u8 = 0x3000_0000usize as *mut u8;
// pub const APP_SIZE: usize = 128 * 1024;

pub static API: H7Api = H7Api {
    alloc,
    free,
    panic,
    // IO
    getc,
    putc,
    puts,
};

pub fn get_address(data: &[u8]) -> AppEntryPoint {
    unsafe {
        let addr = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let ptr = addr as *const ();
        core::mem::transmute(ptr)
    }
}

pub fn check_address(addr: AppEntryPoint) -> Result<&'static str, &'static str> {
    let ptr = addr as *const u8;
    let masked_addr = (ptr as usize) & !THUMB_MASK;
    match (
        // SAFTEY: Pointers are never dereferenced and we're far from overflows.
        unsafe {
            ptr.ge(&(APP_START.add(4) as *const _))
                && ptr.lt(&(APP_START.add(APP_SIZE) as *const _))
        },
        (ptr as usize) & THUMB_MASK == 1,    // Thumb?
        masked_addr % THUMB_ADDR_ALIGN == 0, // Valid Thumb alignment?
        masked_addr % ARM_ADDR_ALIGN == 0,   // Valid ARM alignment?
    ) {
        // (in_range, is_thumb, is_thumb_aligned, is_arm_aligned) => {}
        (true, true, true, _) => Ok("valid thumb"),
        (true, false, true, true) => Ok("valid arm"),
        (true, ..) => Err("invalid"),
        (false, ..) => Err("out of range"),
    }
}

pub fn app_slice() -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(APP_START, APP_SIZE) }
}

pub fn verify_app(slice: &[u8]) -> Result<u32, u32> {
    let len = slice.len();
    let calculated_crc = utils::interrupt_free(|cs| utils::crc(cs, &slice[..(len - 4)]));
    let provided_crc = u32::from_be_bytes([
        slice[len - 4],
        slice[len - 3],
        slice[len - 2],
        slice[len - 1],
    ]);
    if calculated_crc == provided_crc {
        Ok(calculated_crc)
    } else {
        Err(calculated_crc)
    }
}

pub fn print_info<W: core::fmt::Write>(w: &mut W, data: &[u8]) -> core::fmt::Result {
    let crc = verify_app(data);
    let addr = get_address(data);
    writeln!(
        w,
        "Address: {addr:p} ({addr_check}), CRC: 0x{crc:08x} ({crc_check}), Size: 0x{size:x}",
        addr_check = crate::utils::into_ok_or_err(check_address(addr)),
        crc = crate::utils::into_ok_or_err(crc),
        crc_check = if crc.is_ok() { "passed" } else { "failed" },
        size = data.len()
    )
}

// Keep track of app allocations so that we can free leaked application memory
static APP_ALLOCATIONS: Mutex<RefCell<heapless::FnvIndexMap<usize, core::alloc::Layout, 128>>> =
    Mutex::new(RefCell::new(heapless::FnvIndexMap::new()));

pub fn free_leaked() -> usize {
    utils::interrupt_free(|cs| {
        let mut leaked = 0;
        let mut allocations = APP_ALLOCATIONS.borrow(cs).borrow_mut();
        for (ptr, layout) in allocations.iter().map(|(k, v)| ((*k) as *mut u8, *v)) {
            leaked += layout.size();
            unsafe { mem::ALLOCATOR.dealloc(ptr, layout) };
        }
        allocations.clear();
        leaked
    })
}

extern "C" fn alloc(size: usize, align: usize) -> *mut u8 {
    match core::alloc::Layout::from_size_align(size, align) {
        Ok(layout) => utils::interrupt_free(|cs| {
            let ptr = unsafe { mem::ALLOCATOR.alloc(layout) };
            if ptr.is_null() {
                return ptr;
            }
            match APP_ALLOCATIONS
                .borrow(cs)
                .borrow_mut()
                .insert(ptr as usize, layout)
            {
                // Allocator returned a pointer that already exists...
                Ok(Some(_layout)) => panic!("Allocation collision! {:p}", ptr),
                // All good
                Ok(None) => ptr,
                // Insert failed, free allocation and return nullptr
                Err(_) => {
                    unsafe { mem::ALLOCATOR.dealloc(ptr, layout) };
                    core::ptr::null_mut()
                }
            }
        }),
        _ => core::ptr::null_mut(),
    }
}

extern "C" fn free(ptr: *mut u8) {
    utils::interrupt_free(|cs| {
        if let Some(layout) = APP_ALLOCATIONS
            .borrow(cs)
            .borrow_mut()
            .remove(&(ptr as usize))
        {
            unsafe { mem::ALLOCATOR.dealloc(ptr, layout) }
        }
    });
}

extern "C" fn panic(start: *const u8, len: usize) -> ! {
    let s = unsafe { core::slice::from_raw_parts(start, len) };
    match core::str::from_utf8(s) {
        Ok(msg) => panic!("{}", msg),
        _ => panic!("User app paniced with invalid message"),
    }
}

// IO

extern "C" fn getc() -> u8 {
    TERMINAL_INPUT_FIFO.dequeue().unwrap_or(0)
}

extern "C" fn putc(c: u8) -> i32 {
    match write!(TerminalWriter, "{}", c as char) {
        Ok(_) => 0,
        _ => -1,
    }
}

extern "C" fn puts(start: *const u8, len: usize) -> i32 {
    let s = unsafe { core::slice::from_raw_parts(start, len) };

    match core::str::from_utf8(s).map(|s| write!(TerminalWriter, "{s}")) {
        Ok(Ok(_)) => 0,
        _ => -1,
    }
}
