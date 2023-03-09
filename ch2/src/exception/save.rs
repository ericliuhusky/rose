use super::context::Context;
use super::restore::restore_context;
use core::arch::global_asm;
use super::exception_handler;

global_asm!(include_str!("save.s"));

extern "C" {
    fn __save();
    fn CONTEXT_START_ADDR();
}

#[no_mangle]
fn save_context() {
    unsafe {
        let addr = *(CONTEXT_START_ADDR as *const usize);
        let cx = &mut *(addr as *mut Context);
        cx.sepc = riscv_register::sepc::read();
        cx.x[2] = riscv_register::sscratch::read();
        exception_handler();
        restore_context();
    }
}
