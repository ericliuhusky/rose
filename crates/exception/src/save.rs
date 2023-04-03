use crate::TRAP_CONTEXT_ADDR;
use crate::restore::TEMP_CONTEXT;

use super::context::Context;
use super::restore::restore_context;
use core::arch::asm;

extern "C" {
    fn exception_handler();
}

#[link_section = ".text.trampoline"]
#[no_mangle]
fn save_context(cx_user_va: usize) {
    unsafe {
        let cx = &mut *(cx_user_va as *mut Context);
        cx.sepc = riscv_register::sepc::read();
        cx.x[10] = riscv_register::sscratch::read();
        #[cfg(feature = "memory_set")]
        super::memory_set::switch_kernel();
        let cx = &mut *(TRAP_CONTEXT_ADDR as *mut Context);
        for i in 0..32 {
            cx.x[i] = TEMP_CONTEXT.x[i];
        }
        cx.sepc = TEMP_CONTEXT.sepc;
        exception_handler();
    }
}

#[link_section = ".text.trampoline"]
#[repr(align(8))]
#[naked]
pub extern "C" fn save() {
    unsafe {
        asm!(
            "
            csrw sscratch, a0

            la a0, TEMP_CONTEXT
            
            sd x1, 1*8(a0)
            sd x2, 2*8(a0)
            sd x3, 3*8(a0)
            sd x4, 4*8(a0)
            sd x5, 5*8(a0)
            sd x6, 6*8(a0)
            sd x7, 7*8(a0)
            sd x8, 8*8(a0)
            sd x9, 9*8(a0)
            sd x11, 11*8(a0)
            sd x12, 12*8(a0)
            sd x13, 13*8(a0)
            sd x14, 14*8(a0)
            sd x15, 15*8(a0)
            sd x16, 16*8(a0)
            sd x17, 17*8(a0)
            sd x18, 18*8(a0)
            sd x19, 19*8(a0)
            sd x20, 20*8(a0)
            sd x21, 21*8(a0)
            sd x22, 22*8(a0)
            sd x23, 23*8(a0)
            sd x24, 24*8(a0)
            sd x25, 25*8(a0)
            sd x26, 26*8(a0)
            sd x27, 27*8(a0)
            sd x28, 28*8(a0)
            sd x29, 29*8(a0)
            sd x30, 30*8(a0)
            sd x31, 31*8(a0)


            ld sp, KERNEL_STACK_TOP

            call save_context
        ",
            options(noreturn)
        )
    }
}
