use crate::segment::CONTEXT_START_ADDR;
use super::context::Context;
use core::arch::global_asm;
use super::exception_handler;

global_asm!(include_str!("save.s"));

extern "C" {
    fn __save();
}

#[no_mangle]
fn save_context() {
    use core::arch::asm;
    unsafe {
        let cx = &mut *(CONTEXT_START_ADDR as *mut Context);
        let mut t: usize;
        asm!("csrr {}, sepc", out(reg) t);
        cx.sepc = t;
        let mut t2: usize;
        asm!("csrr {}, sscratch", out(reg) t2);
        cx.x[2] = t2;
        exception_handler();
    }
}
