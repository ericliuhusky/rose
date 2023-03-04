use 系统调用_输出::{系统调用_输出, sys_putchar};
use 系统调用_终止::系统调用_终止;

const 系统调用标识_输出: usize = 1;
const 系统调用标识_终止: usize = 2;
const SYS_PUTCHAR: usize = 9;

pub fn 系统调用(系统调用标识: usize, 参数: [usize; 3]) -> isize {
    match 系统调用标识 {
        系统调用标识_输出 => {
            系统调用_输出(参数[0] as *const u8, 参数[1])
        },
        系统调用标识_终止 => 系统调用_终止(参数[0] as i32),
        SYS_PUTCHAR => sys_putchar(参数[0]),
        _ => {
            println!("[kernel] Unsupported syscall_id: {}", 系统调用标识);
            -1
        }
    }
}

mod 系统调用_输出 {
    use crate::task::任务管理器;

    pub fn 系统调用_输出(字节数组指针: *const u8, 字节数组长度: usize) -> isize {
        let 当前任务的地址空间 = &任务管理器::当前任务().地址空间;
        let va_range = 字节数组指针 as usize..字节数组指针 as usize + 字节数组长度;
        let 字节数组 = 当前任务的地址空间.读取字节数组(va_range);
        let 字符串 = core::str::from_utf8(&字节数组).unwrap();
        print!("{}", 字符串);
        字节数组长度 as isize
    }

    pub fn sys_putchar(c: usize) -> isize {
        sbi_call::putchar(c);
        c as isize
    }
}

mod 系统调用_终止 {
    use crate::task::任务管理器;

    pub fn 系统调用_终止(代码: i32) -> isize {
        println!("[kernel] Application exited with code {}", 代码);
        任务管理器::终止并运行下一个任务();
        -1
    }
}
