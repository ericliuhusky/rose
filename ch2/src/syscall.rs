const SYS_EXIT: usize = 2;
const SYS_PUTCHAR: usize = 9;

pub fn sys_func(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYS_EXIT => exit(args[0] as isize),
        SYS_PUTCHAR => putchar(args[0]),
        _ => {
            println!("[kernel] Unsupported syscall_id: {}", id);
            -1
        }
    }
}

fn putchar(c: usize) -> isize {
    sbi_call::putchar(c);
    c as isize
}

use crate::batch::应用管理器;

fn exit(exit_code: isize) -> isize {
    println!("[kernel] Application exited with code {}", exit_code);
    应用管理器::运行下一个应用();
    -1
}
