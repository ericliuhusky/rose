#![no_std]

use core::arch::asm;

#[inline(always)]
fn sbi_call(id: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") id
        );
    }
    ret
}

const SBI_PUTCHAR: usize = 1;

pub fn putchar(c: usize) {
    sbi_call(SBI_PUTCHAR, c, 0, 0);
}
