use crate::exception::Context;
use alloc::vec::Vec;

extern "C" {
    fn ekernel();
}

#[no_mangle]
static mut KERNEL_STACK_TOP: usize = 0;
pub static mut CONTEXT_START_ADDRS: Vec<usize> = Vec::new();
#[no_mangle]
pub static mut CONTEXT_START_ADDR: usize = 0;
pub static mut APP_START_ADDR: usize = 0;

pub fn init() {
    let n = loader::read_app_num();
    unsafe {
        KERNEL_STACK_TOP = ekernel as usize + 0x2000;
        for i in 0..n {
            CONTEXT_START_ADDRS.push(KERNEL_STACK_TOP +  i * core::mem::size_of::<Context>());
        }
        APP_START_ADDR = CONTEXT_START_ADDRS[n-1] + core::mem::size_of::<Context>();
    }
}
