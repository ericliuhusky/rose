#![no_std]

use core::arch::asm;

#[inline(always)]
fn sbi_call(id: usize, args: [usize; 3]) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

const SBI_SET_TIMER: usize = 0;
const SBI_PUTCHAR: usize = 1;
const SBI_GETCHAR: usize = 2;
const SBI_SHUTDOWN: usize = 8;


pub fn putchar(c: usize) {
    sbi_call(SBI_PUTCHAR, [c, 0, 0]);
}

pub fn getchar() -> usize {
    sbi_call(SBI_GETCHAR, [0, 0, 0])
}

pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, [0, 0, 0]);
    panic!("shutdown")
}

pub fn set_timer(time: usize) {
    sbi_call(SBI_SET_TIMER, [time, 0, 0]);
}
