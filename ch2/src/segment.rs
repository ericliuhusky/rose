use exception::context::Context;

extern "C" {
    fn ekernel();
}

#[no_mangle]
static mut KERNEL_STACK_TOP: usize = 0;
#[no_mangle]
pub static mut CONTEXT_START_ADDR: usize = 0;
pub static mut APP_START_ADDR: usize = 0;
pub static mut APP_END_ADDR: usize = 0;

pub fn init() {
    unsafe {
        KERNEL_STACK_TOP = ekernel as usize + 0x2000;
        CONTEXT_START_ADDR = KERNEL_STACK_TOP;
        APP_START_ADDR = CONTEXT_START_ADDR + core::mem::size_of::<Context>();
    }
}
