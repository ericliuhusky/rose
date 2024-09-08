use super::context::Context;
use super::TRAP_CONTEXT_ADDR;
use core::arch::asm;

#[link_section = ".text.trampoline"]
#[inline(never)]
pub fn restore_context(cx_ptr: *const Context, user_satp: usize) {
    unsafe {
        for i in 0..32 {
            TEMP_CONTEXT.x[i] = (*cx_ptr).x[i];
        }
        TEMP_CONTEXT.sepc = (*cx_ptr).sepc;
        TRAP_CONTEXT_ADDR = cx_ptr as usize;
        super::memory_set::switch_user(user_satp);
        riscv::register::sepc::write(TEMP_CONTEXT.sepc);
        restore(&TEMP_CONTEXT);
    }
}

#[link_section = ".text.trampoline"]
pub static mut TEMP_CONTEXT: Context = Context {
    x: [0; 32],
    sepc: 0,
};

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
unsafe extern "C" fn restore(cx: &Context) {
    asm!(
        load_registers_from_a0!(),
        "
        ld a0, 10*8(a0)

        sret
    ",
        options(noreturn)
    )
}
