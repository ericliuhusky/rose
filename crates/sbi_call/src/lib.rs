#![no_std]

use core::arch::asm;

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize) -> (usize, usize) {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
        );
    }
    (error, value)
}

const SBI_PUTCHAR: usize = 1;
const SBI_GETCHAR: usize = 2;

const EID_SRST: usize = eid_from_str("SRST");
const SYSTEM_RESET: usize = 0;
const SHUTDOWN: usize = 0;
const NOREASON: usize = 0;

const EID_TIME: usize = eid_from_str("TIME");
const SET_TIMER: usize = 0;

const fn eid_from_str(name: &str) -> usize {
    let bytes = name.as_bytes();
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
}

pub fn putchar(c: usize) {
    sbi_call(SBI_PUTCHAR, 0, c, 0);
}

pub fn getchar() -> usize {
    sbi_call(SBI_GETCHAR, 0, 0, 0).0
}

pub fn shutdown() -> ! {
    sbi_call(EID_SRST, SYSTEM_RESET, SHUTDOWN, NOREASON);
    unreachable!()
}

pub fn set_timer(time: usize) {
    sbi_call(EID_TIME, SET_TIMER, time, 0);
}
