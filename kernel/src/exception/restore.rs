use super::context::Context;
use super::TRAP_CONTEXT_ADDR;
use core::arch::asm;

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
pub extern "C" fn restore_context(cx_ptr: *const Context, user_satp: usize) {
    unsafe {
        asm!(
            "csrw sscratch, a1", // write user_satp to sscratch
            "
            la t0, {TRAP_CONTEXT_ADDR}
            sd a0, (t0)
            ", // TRAP_CONTEXT_ADDR = cx_ptr as usize;
            "
            ld t0, 32*8(a0)
            csrw sepc, t0
            ", // write context sepc to sepc register
            load_registers_from_a0!(),
            "ld a0, 10*8(a0)",
            "
            csrrw a0, sscratch, a0
            csrw satp, a0
            sfence.vma
            csrrw a0, sscratch, a0
            ", // switch to user memory set
            "sret",
            TRAP_CONTEXT_ADDR = sym TRAP_CONTEXT_ADDR,
            options(noreturn)
        )
    }
}
