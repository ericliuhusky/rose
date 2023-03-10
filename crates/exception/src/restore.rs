use super::context::Context;
use core::arch::global_asm;

global_asm!(include_str!("restore.s"));

extern "C" {
    fn __restore();
    fn CONTEXT_START_ADDR();
}

#[link_section = ".text.trampoline"]
#[inline(never)]
pub fn restore_context() {
    unsafe {
        #[cfg(feature = "memory_set")]
        super::memory_set::switch_user();
        let addr = *(CONTEXT_START_ADDR as *const usize);
        let cx = &*(addr as *const Context);
        riscv_register::sepc::write(cx.sepc);
        __restore();
    }
}
