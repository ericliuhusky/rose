use crate::segment::CONTEXT_START_ADDR;
use super::context::Context;
use core::arch::global_asm;

global_asm!(include_str!("restore.s"));

extern "C" {
    fn __restore();
}

pub fn restore_context() {
    use core::arch::asm;
    unsafe {
        let cx = &*(CONTEXT_START_ADDR as *const Context);
        asm!("csrw sepc, {}", in(reg) cx.sepc);
        __restore();
    }
}
