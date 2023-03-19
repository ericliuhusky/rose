#![no_std]

pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_EXIT: usize = 2;
pub const SYS_YIELD: usize = 3;
pub const SYS_GET_TIME: usize = 4;
pub const SYS_GETPID: usize = 5;
pub const SYS_FORK: usize = 6;
pub const SYS_EXEC: usize = 7;
pub const SYS_WAITPID: usize = 8;
pub const SYS_PUTCHAR: usize = 9;
pub const SYS_GETCHAR: usize = 10;
pub const SYS_OPEN: usize = 11;
pub const SYS_CLOSE: usize = 12;