use crate::segment::CONTEXT_START_ADDR;
use super::context::Context;
use core::arch::global_asm;

global_asm!(include_str!("restore.s"));

extern "C" {
    fn __restore();
}

pub fn restore_context() {
    unsafe {
        let cx = &*(CONTEXT_START_ADDR as *const Context);
        riscv_register::sepc::write(cx.sepc);
        __restore();
    }
}
