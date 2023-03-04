#![no_std]

use core::arch::asm;

#[inline(always)]
fn sys_call(id: usize, args: [usize; 3]) -> isize {
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

const SYS_PUTCHAR: usize = 9;
const SYS_EXIT: usize = 2;

pub fn putchar(c: usize) {
    sys_call(SYS_PUTCHAR, [c, 0, 0]);
}

pub fn exit(exit_code: isize) -> ! {
    sys_call(SYS_EXIT, [exit_code as usize, 0, 0]);
    panic!("exit")
}
