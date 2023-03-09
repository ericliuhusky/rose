use super::context::Context;
use core::arch::global_asm;

global_asm!(include_str!("restore.s"));

extern "C" {
    fn __restore();
    fn CONTEXT_START_ADDR();
}

pub fn restore_context() {
    unsafe {
        let addr = *(CONTEXT_START_ADDR as *const usize);
        let cx = &*(addr as *const Context);
        riscv_register::sepc::write(cx.sepc);
        __restore();
    }
}
