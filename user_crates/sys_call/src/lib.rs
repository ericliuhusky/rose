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

const SYS_READ: usize = 0;
const SYS_WRITE: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_YIELD: usize = 3;
const SYS_GET_TIME: usize = 4;
const SYS_GETPID: usize = 5;
const SYS_FORK: usize = 6;
const SYS_EXEC: usize = 7;
const SYS_WAITPID: usize = 8;
const SYS_PUTCHAR: usize = 9;
const SYS_GETCHAR: usize = 10;

pub fn read(buffer: &mut [u8]) -> isize {
    sys_call(SYS_READ, [buffer as *mut [u8] as *mut u8 as usize, buffer.len(), 0])
}

pub fn putchar(c: usize) {
    sys_call(SYS_PUTCHAR, [c, 0, 0]);
}

pub fn getchar() -> isize {
    sys_call(SYS_GETCHAR, [0, 0, 0])
}

pub fn exit(exit_code: isize) -> ! {
    sys_call(SYS_EXIT, [exit_code as usize, 0, 0]);
    panic!("exit")
}

pub fn yield_() -> isize {
    sys_call(SYS_YIELD, [0, 0, 0])
}

pub fn get_time() -> isize {
    sys_call(SYS_GET_TIME, [0, 0, 0])
}

pub fn getpid() -> isize {
    sys_call(SYS_GETPID, [0, 0, 0])
}

pub fn fork() -> isize {
    sys_call(SYS_FORK, [0, 0, 0])
}

pub fn exec(path: &str) -> isize {
    sys_call(SYS_EXEC, [path.as_ptr() as usize, path.len(), 0])
}

pub fn waitpid(pid: isize, exit_code: *mut i32) -> isize {
    sys_call(SYS_WAITPID, [pid as usize, exit_code as usize, 0])
}
