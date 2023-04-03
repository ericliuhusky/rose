use crate::TRAP_CONTEXT_ADDR;
use super::context::Context;
use core::arch::asm;

#[link_section = ".text.trampoline"]
#[inline(never)]
pub fn restore_context(cx_user_va: usize, user_satp: usize) {
    unsafe {
        let cx = &*(cx_user_va as *const Context);
        for i in 0..32 {
            TEMP_CONTEXT.x[i] = cx.x[i];
        }
        TEMP_CONTEXT.sepc = cx.sepc;
        TRAP_CONTEXT_ADDR = cx_user_va;
        #[cfg(feature = "memory_set")]
        super::memory_set::switch_user(user_satp);
        riscv_register::sepc::write(TEMP_CONTEXT.sepc);
        restore(&TEMP_CONTEXT);
    }
}

#[no_mangle]
#[link_section = ".text.trampoline"]
pub static mut TEMP_CONTEXT: Context = Context { x: [0; 32], sepc: 0 };

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
extern "C" fn restore(cx: &Context) {
    unsafe {
        asm!(
            "
            ld x1, 1*8(a0)
            ld x2, 2*8(a0)
            ld x3, 3*8(a0)
            ld x4, 4*8(a0)
            ld x5, 5*8(a0)
            ld x6, 6*8(a0)
            ld x7, 7*8(a0)
            ld x8, 8*8(a0)
            ld x9, 9*8(a0)
            ld x11, 11*8(a0)
            ld x12, 12*8(a0)
            ld x13, 13*8(a0)
            ld x14, 14*8(a0)
            ld x15, 15*8(a0)
            ld x16, 16*8(a0)
            ld x17, 17*8(a0)
            ld x18, 18*8(a0)
            ld x19, 19*8(a0)
            ld x20, 20*8(a0)
            ld x21, 21*8(a0)
            ld x22, 22*8(a0)
            ld x23, 23*8(a0)
            ld x24, 24*8(a0)
            ld x25, 25*8(a0)
            ld x26, 26*8(a0)
            ld x27, 27*8(a0)
            ld x28, 28*8(a0)
            ld x29, 29*8(a0)
            ld x30, 30*8(a0)
            ld x31, 31*8(a0)

            ld a0, 10*8(a0)

            sret
        ",
            options(noreturn)
        )
    }
}
