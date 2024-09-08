use super::context::Context;
use super::exception_handler::exception_handler;
use super::restore::TEMP_CONTEXT;
use super::TRAP_CONTEXT_ADDR;
use core::arch::asm;

#[link_section = ".text.trampoline"]
fn save_context(cx_user_va: usize) {
    unsafe {
        let cx = &mut *(cx_user_va as *mut Context);
        cx.sepc = riscv::register::sepc::read();
        super::memory_set::switch_kernel();
        let cx = &mut *(TRAP_CONTEXT_ADDR as *mut Context);
        for i in 0..32 {
            cx.x[i] = TEMP_CONTEXT.x[i];
        }
        cx.sepc = TEMP_CONTEXT.sepc;
        exception_handler();
    }
}

const KERNEL_STACK_TOP: usize = 0x87800000;

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
pub unsafe extern "C" fn save() {
    asm!(
        write_a0_to_scratch!(s),
        "la a0, {TEMP_CONTEXT}",
        store_registers_to_a0!(),
        read_scratch_and_store_to_a0!(s),
        "
        li sp, {KERNEL_STACK_TOP}

        call {save_context}
        ",
        KERNEL_STACK_TOP = const KERNEL_STACK_TOP,
        TEMP_CONTEXT = sym TEMP_CONTEXT,
        save_context = sym save_context,
        options(noreturn)
    )
}
