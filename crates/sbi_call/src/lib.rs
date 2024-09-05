#![no_std]

use core::arch::asm;

#[inline(always)]
fn sbi_call(id: usize, args: [usize; 3]) -> usize {
    let ret;
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

const SET_TIMER: usize = 0;
const GET_TIME: usize = 1;
const SHUTDOWN: usize = 2;
const PUTCHAR: usize = 3;
const GETCHAR: usize = 4;

pub fn putchar(c: usize) {
    sbi_call(PUTCHAR, [c, 0, 0]);
}

pub fn getchar() -> usize {
    sbi_call(GETCHAR, [0, 0, 0])
}

pub fn shutdown() -> ! {
    sbi_call(SHUTDOWN, [0, 0, 0]);
    unreachable!()
}

pub fn set_timer(time: usize) {
    sbi_call(SET_TIMER, [time, 0, 0]);
}

pub fn get_time() -> usize {
    sbi_call(GET_TIME, [0, 0, 0])
}
