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
        cx.x[2] = riscv_register::sscratch::read();
        #[cfg(feature = "memory_set")]
        super::memory_set::switch_kernel();
        exception_handler();
    }
}

#[link_section = ".text.trampoline"]
#[naked]
pub extern "C" fn save() {
    unsafe {
        asm!(
            "
            csrrw sp, sscratch, sp
            
            sd x1, 1*8(sp)
            sd x3, 3*8(sp)
            sd x4, 4*8(sp)
            sd x5, 5*8(sp)
            sd x6, 6*8(sp)
            sd x7, 7*8(sp)
            sd x8, 8*8(sp)
            sd x9, 9*8(sp)
            sd x10, 10*8(sp)
            sd x11, 11*8(sp)
            sd x12, 12*8(sp)
            sd x13, 13*8(sp)
            sd x14, 14*8(sp)
            sd x15, 15*8(sp)
            sd x16, 16*8(sp)
            sd x17, 17*8(sp)
            sd x18, 18*8(sp)
            sd x19, 19*8(sp)
            sd x20, 20*8(sp)
            sd x21, 21*8(sp)
            sd x22, 22*8(sp)
            sd x23, 23*8(sp)
            sd x24, 24*8(sp)
            sd x25, 25*8(sp)
            sd x26, 26*8(sp)
            sd x27, 27*8(sp)
            sd x28, 28*8(sp)
            sd x29, 29*8(sp)
            sd x30, 30*8(sp)
            sd x31, 31*8(sp)

            mv a0, sp

            ld sp, KERNEL_STACK_TOP

            call save_context
        ",
            options(noreturn)
        )
    }
}
