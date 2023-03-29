use sys_func::SysFunc;
use 系统调用_时钟计数器::get_time;
use 系统调用_终止::exit;
use 系统调用_让出时间片::yield_;
use 系统调用_读取::{getchar, read};
use 系统调用_输出::{putchar, write};
use 系统调用_进程::{exec, fork, getpid, waitpid};

pub struct SysFuncImpl;

impl SysFunc for SysFuncImpl {
    fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        read(fd, buf, len)
    }
    fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        write(fd, buf, len)
    }
    fn exit(exit_code: i32) -> isize {
        exit(exit_code)
    }
    fn yield_() -> isize {
        yield_()
    }
    fn get_time() -> isize {
        get_time()
    }
    fn getpid() -> isize {
        getpid()
    }
    fn fork() -> isize {
        fork()
    }
    fn exec(path: *const u8, len: usize) -> isize {
        exec(path, len)
    }
    fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        waitpid(pid, exit_code_ptr)
    }
    fn putchar(c: usize) -> isize {
        putchar(c)
    }
    fn getchar() -> isize {
        getchar()
    }
    fn open(path: *const u8, len: usize, create: u32) -> isize {
        open(path, len, create)
    }
    fn close(fd: usize) -> isize {
        close(fd)
    }
    fn pipe(pipe_fd: *mut usize) -> isize {
        pipe(pipe_fd)
    }
}

mod 系统调用_输出 {
    use crate::task::任务管理器;
    use page_table::VA;

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        let task = 任务管理器::当前任务();
        if fd >= task.fd_table.len() {
            return -1;
        }
        if let Some(file) = &task.fd_table[fd] {
            let file = file.clone();
            let buf = 任务管理器::当前任务()
                .地址空间
                .page_table
                .translate_buffer(buf as usize, len);
            file.write(buf) as isize
        } else {
            -1
        }
    }

    pub fn putchar(c: usize) -> isize {
        sbi_call::putchar(c);
        c as isize
    }
}

mod 系统调用_终止 {
    use crate::task::任务管理器;

    pub fn exit(exit_code: i32) -> isize {
        println!("[kernel] Application exited with code {}", exit_code);
        任务管理器::终止并运行下一个任务(exit_code);
        -1
    }
}

mod 系统调用_读取 {
    use crate::task::任务管理器;
    use page_table::VA;

    pub fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        let task = 任务管理器::当前任务();
        if fd >= task.fd_table.len() {
            return -1;
        }
        if let Some(file) = &task.fd_table[fd] {
            let file = file.clone();
            let buf = task
                .地址空间
                .page_table
                .translate_buffer(buf as usize, len);
            file.read(buf) as isize
        } else {
            -1
        }
    }

    pub fn getchar() -> isize {
        sbi_call::getchar() as isize
    }
}

mod 系统调用_让出时间片 {
    use crate::task::任务管理器;

    pub fn yield_() -> isize {
        任务管理器::暂停并运行下一个任务();
        0
    }
}

mod 系统调用_时钟计数器 {
    use crate::timer::读取时钟计数器的毫秒值;

    pub fn get_time() -> isize {
        读取时钟计数器的毫秒值() as isize
    }
}

mod 系统调用_进程 {
    use crate::task::task::任务状态;
    use crate::task::任务管理器;
    use alloc::string::String;

    pub fn getpid() -> isize {
        任务管理器::当前任务().进程标识符.0 as isize
    }

    pub fn fork() -> isize {
        任务管理器::可变当前任务(|mut 任务| {
            let 新任务 = 任务.fork();
            let 上下文 = 新任务.borrow().地址空间.陷入上下文();
            上下文.x[10] = 0;
            let 新任务进程标识符 = 新任务.borrow().进程标识符.0;
            任务管理器::添加任务(新任务);
            新任务进程标识符 as isize
        })
    }

    use crate::fs::open_file;

    pub fn exec(path: *const u8, len: usize) -> isize {
        let 应用名称 = 任务管理器::当前任务()
            .地址空间
            .read_str(path as usize, len);
        if let Some(elf_inode) = open_file(&应用名称, false) {
            let elf_data = elf_inode.read_all();
            任务管理器::可变当前任务(|mut 任务| {
                任务.exec(&elf_data);
            });
            0
        } else {
            -1
        }
    }

    pub fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        任务管理器::可变当前任务(|mut task| {
            if !task
                .子进程列表
                .iter()
                .any(|p| pid == -1 || pid as usize == p.borrow().进程标识符.0)
            {
                return -1;
            }

            let pair = task.子进程列表.iter().enumerate().find(|(_, p)| {
                let p = p.borrow();
                p.状态 == 任务状态::终止 && (pid == -1 || pid as usize == p.进程标识符.0)
            });
            if let Some((idx, _)) = pair {
                let child = task.子进程列表.remove(idx);
                let found_pid = child.borrow().进程标识符.0;
                // TODO: 终止代码
                // let exit_code = child.borrow().终止代码;
                // let refmut = task.memory_set.page_table.translated_refmut(exit_code_ptr);
                // *refmut = exit_code;
                found_pid as isize
            } else {
                -2
            }
        })
    }
}

use crate::fs::open_file;
use crate::task::{task::任务, 任务管理器};
use alloc::string::String;
use page_table::VA;

pub fn open(path: *const u8, len: usize, create: u32) -> isize {
    let task = 任务管理器::当前任务();
    let path = task
        .地址空间
        .read_str(path as usize, len);
    let create = create != 0;
    if let Some(inode) = open_file(path.as_str(), create) {
        drop(task);
        任务管理器::可变当前任务(|mut task| {
            let fd = task.alloc_fd();
            task.fd_table[fd] = Some(inode);
            fd as isize
        })
    } else {
        -1
    }
}

pub fn close(fd: usize) -> isize {
    let task = 任务管理器::当前任务();
    if fd >= task.fd_table.len() {
        return -1;
    }
    if task.fd_table[fd].is_none() {
        return -1;
    }
    drop(task);
    任务管理器::可变当前任务(|mut task| {
        task.fd_table[fd].take();
    });
    0
}

use crate::fs::make_pipe;

pub fn pipe(pipe_fd: *mut usize) -> isize {
    任务管理器::可变当前任务(|mut task| {
        let (pipe_read, pipe_write) = make_pipe();
        let read_fd = task.alloc_fd();
        task.fd_table[read_fd] = Some(pipe_read);
        let write_fd = task.alloc_fd();
        task.fd_table[write_fd] = Some(pipe_write);
        let pipe_fd = task.地址空间.page_table.translate_type::<[usize; 2]>(pipe_fd as usize);
        pipe_fd[0] = read_fd;
        pipe_fd[1] = write_fd;
        0
    })
}