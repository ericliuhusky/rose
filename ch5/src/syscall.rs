use 系统调用_输出::系统调用_输出;
use 系统调用_终止::系统调用_终止;
use 系统调用_读取::系统调用_读取;
use 系统调用_让出时间片::系统调用_让出时间片;
use 系统调用_时钟计数器::系统调用_读取时钟计数器的毫秒值;
use 系统调用_进程::{getpid, fork, exec, waitpid};
use crate::格式化输出并换行;

const 系统调用标识_读取: usize = 0;
const 系统调用标识_输出: usize = 1;
const 系统调用标识_终止: usize = 2;
const 系统调用标识_让出时间片: usize = 3;
const 系统调用标识_读取时钟计数器的毫秒值: usize = 4;
const 系统调用标识_进程_GETPID: usize = 5;
const 系统调用标识_进程_FORK: usize = 6;
const 系统调用标识_进程_EXEC: usize = 7;
const 系统调用标识_进程_WAITPID: usize = 8;

pub fn 系统调用(系统调用标识: usize, 参数: [usize; 3]) -> isize {
    match 系统调用标识 {
        系统调用标识_读取 => {
            系统调用_读取(参数[0] as *const u8, 参数[1])
        },
        系统调用标识_输出 => {
            系统调用_输出(参数[0] as *const u8, 参数[1])
        },
        系统调用标识_终止 => 系统调用_终止(参数[0] as i32),
        系统调用标识_让出时间片 => 系统调用_让出时间片(),
        系统调用标识_读取时钟计数器的毫秒值 => 系统调用_读取时钟计数器的毫秒值(),
        系统调用标识_进程_GETPID => getpid(),
        系统调用标识_进程_FORK => fork(),
        系统调用标识_进程_EXEC => exec(参数[0] as *const u8, 参数[1]),
        系统调用标识_进程_WAITPID => waitpid(参数[0] as isize, 参数[1] as *mut i32),
        _ => {
            格式化输出并换行!("[kernel] Unsupported syscall_id: {}", 系统调用标识);
            -1
        }
    }
}

mod 系统调用_输出 {
    use crate::输出::输出;
    use crate::task::任务管理器;

    pub fn 系统调用_输出(字节数组指针: *const u8, 字节数组长度: usize) -> isize {
        let va_range = 字节数组指针 as usize..字节数组指针 as usize + 字节数组长度;
        let 字节数组 = 任务管理器::当前任务().地址空间.读取字节数组(va_range);
        let 字符串 = core::str::from_utf8(&字节数组).unwrap();
        输出(字符串);
        字节数组长度 as isize
    }
}

mod 系统调用_终止 {
    use crate::task::任务管理器;
    use crate::格式化输出并换行;

    pub fn 系统调用_终止(代码: i32) -> isize {
        格式化输出并换行!("[kernel] Application exited with code {}", 代码);
        任务管理器::终止并运行下一个任务(代码);
        -1
    }
}

mod 系统调用_读取 {
    use core::arch::asm;
    use crate::task::任务管理器;

    pub fn 系统调用_读取(字节数组指针: *const u8, 字节数组长度: usize) -> isize {
        let mut 字符: usize;
        unsafe {
            asm!(
                "ecall",
                out("x10") 字符,
                in("x17") 2
            );
        }
        let 字符数组 = [字符 as u8; 1];
        let 虚拟地址范围 = 字节数组指针 as usize..字节数组指针 as usize + 字节数组长度;
        任务管理器::当前任务().地址空间.写入字节数组(虚拟地址范围, &字符数组);
        1
    }
}

mod 系统调用_让出时间片 {
    use crate::task::任务管理器;

    pub fn 系统调用_让出时间片() -> isize {
        任务管理器::暂停并运行下一个任务();
        0
    }
}

mod 系统调用_时钟计数器 {
    use crate::timer::读取时钟计数器的毫秒值;

    pub fn 系统调用_读取时钟计数器的毫秒值() -> isize {
        读取时钟计数器的毫秒值() as isize
    }
}

mod 系统调用_进程 {
    use alloc::string::String;
    use crate::task::任务管理器;
    use crate::loader::通过名称读取应用数据;

    pub fn getpid() -> isize {
        任务管理器::当前任务().进程标识符.0 as isize
    }

    pub fn fork() -> isize {
        任务管理器::可变当前任务(|mut 任务| {
            crate::格式化输出并换行!("fuck1");
            let 新任务 = 任务.fork();
            let 上下文 = 新任务.borrow().地址空间.陷入上下文();
            上下文.通用寄存器[10] = 0;
            let 新任务进程标识符 = 新任务.borrow().进程标识符.0;
            任务管理器::添加任务(新任务);
            新任务进程标识符 as isize
        })
    }

    pub fn exec(path: *const u8, len: usize) -> isize {
        let 虚拟地址范围 = path as usize..path as usize + len;
        let 应用名称: String = 任务管理器::当前任务().地址空间
            .读取字节数组(虚拟地址范围)
            .iter()
            .map(|字节| *字节 as char)
            .collect();
        let elf文件数据 = 通过名称读取应用数据(&应用名称);
        任务管理器::可变当前任务(|mut 任务| {
            任务.exec(elf文件数据);
        });
        0
    }

    pub fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        // if pid == -1 {
        //     if !任务管理器::当前任务().子进程列表.is_empty() {
        //         if let Some((子进程索引, 子进程)) = 任务管理器::当前任务().子进程列表.iter().enumerate().find(|(_, 子进程)| {
        //             子进程.borrow().状态 == 任务状态::终止
        //         }) {
        //             任务管理器::可变当前任务(|mut 任务| {
        //                 任务.子进程列表.remove(子进程索引)
        //             });
        //             let 子进程标识符 = 子进程.borrow().进程标识符.0;
        //             子进程标识符 as isize
        //         } else {
        //             -2
        //         }
        //     } else {
        //         -1
        //     }
        // } else {
        //     if let Some((子进程索引, 子进程)) = 任务管理器::当前任务().子进程列表.iter().enumerate().find(|(_, 子进程)| {
        //         子进程.borrow().进程标识符.0 == pid as usize
        //     }) {
        //         if 子进程.borrow().状态 == 任务状态::终止 {
        //             任务管理器::可变当前任务(|mut 任务| {
        //                 任务.子进程列表.remove(子进程索引)
        //             });
        //             let 子进程标识符 = 子进程.borrow().进程标识符.0;
        //             子进程标识符 as isize
        //         } else {
        //             -2
        //         }
        //     } else {
        //         -1
        //     }
        // }
        -2
    }
}
