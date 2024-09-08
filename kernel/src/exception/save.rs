use super::exception_handler::exception_handler;
use super::memory_set::KERNEL_SATP;
use super::TRAP_CONTEXT_ADDR;
use core::arch::asm;

const KERNEL_STACK_TOP: usize = 0x87800000;

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
pub unsafe extern "C" fn save() {
    asm!(
        write_a0_to_scratch!(s),
        "
        ld a0, {KERNEL_SATP}
        csrw satp, a0
        sfence.vma
        ", // switch to kernel memory set
        "ld a0, {TRAP_CONTEXT_ADDR}",
        store_registers_to_a0!(),
        read_scratch_and_store_to_a0!(s),
        "li sp, {KERNEL_STACK_TOP}",
        "
        csrr t0, sepc
        sd t0, 32*8(a0)
        ", // read sepc from sepc register, and store to context's sepc
        "
        call {exception_handler}
        ",
        KERNEL_STACK_TOP = const KERNEL_STACK_TOP,
        TRAP_CONTEXT_ADDR = sym TRAP_CONTEXT_ADDR,
        exception_handler = sym exception_handler,
        KERNEL_SATP = sym KERNEL_SATP,
        options(noreturn)
    )
}
