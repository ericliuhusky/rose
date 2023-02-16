mod sys_exit;
mod sys_puts;
mod sys_yield;

use sys_exit::sys_exit;
use sys_puts::sys_puts;
use sys_yield::sys_yield;

const SYSCALL_PUTS: usize = 1;
const SYSCALL_EXIT: usize = 2;
const SYSCALL_YIELD: usize = 3;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_PUTS => {
            sys_puts(args[0] as *const u8, args[1])
        },
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        _ => {
            println!("[kernel] Unsupported syscall_id: {}", syscall_id);
            -1
        }
    }
}
