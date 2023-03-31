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
    use crate::task::{TaskManager, current_task};
    use page_table::VA;

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        let task = current_task();
        let fd_table = &task.borrow().fd_table;
        if fd >= fd_table.len() {
            return -1;
        }
        if let Some(file) = &fd_table[fd] {
            let file = file.clone();
            let buf = task.borrow()
                .memory_set
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
    use crate::task::{TaskManager, exit_and_run_next};

    pub fn exit(exit_code: i32) -> isize {
        println!("[kernel] Application exited with code {}", exit_code);
        exit_and_run_next(exit_code);
        -1
    }
}

mod 系统调用_读取 {
    use crate::task::{TaskManager, current_task};
    use page_table::VA;

    pub fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        let task = current_task();
        let fd_table = &task.borrow().fd_table;
        if fd >= fd_table.len() {
            return -1;
        }
        if let Some(file) = &fd_table[fd] {
            let file = file.clone();
            let buf = task.borrow()
                .memory_set
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
    use crate::task::{TaskManager, suspend_and_run_next};

    pub fn yield_() -> isize {
        suspend_and_run_next();
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
    use crate::task::{TaskManager, current_task, add_task};
    use alloc::string::String;

    pub fn getpid() -> isize {
        current_task().borrow().pid.0 as isize
    }

    pub fn fork() -> isize {
        let task = current_task();
        let mut task = task.borrow_mut();
        let new_task = task.fork();
        let cx = new_task.borrow().memory_set.get_context();
        cx.x[10] = 0;
        let new_task_pid = new_task.borrow().pid.0;
        add_task(new_task);
        new_task_pid as isize
    }

    use crate::fs::open_file;

    pub fn exec(path: *const u8, len: usize) -> isize {
        let task = current_task();
        let 应用名称 = task.borrow()
            .memory_set
            .read_str(path as usize, len);
        if let Some(elf_inode) = open_file(&应用名称, false) {
            let elf_data = elf_inode.read_all();
            let mut task = task.borrow_mut();
            task.exec(&elf_data);
            0
        } else {
            -1
        }
    }

    pub fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        let task = current_task();
        let mut task = task.borrow_mut();
        if !task
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.borrow().pid.0)
        {
            return -1;
        }

        let pair = task.children.iter().enumerate().find(|(_, p)| {
            let p = p.borrow();
            p.is_exited && (pid == -1 || pid as usize == p.pid.0)
        });
        if let Some((idx, _)) = pair {
            let child = task.children.remove(idx);
            let found_pid = child.borrow().pid.0;
            // TODO: 终止代码
            // let exit_code = child.borrow().终止代码;
            // let refmut = task.memory_set.page_table.translated_refmut(exit_code_ptr);
            // *refmut = exit_code;
            found_pid as isize
        } else {
            -2
        }
    }
}

use crate::fs::open_file;
use crate::task::current_task;
use crate::task::{task::Task, TaskManager};
use alloc::string::String;
use page_table::VA;

pub fn open(path: *const u8, len: usize, create: u32) -> isize {
    let task = current_task();
    let path = task.borrow()
        .memory_set
        .read_str(path as usize, len);
    let create = create != 0;
    if let Some(inode) = open_file(path.as_str(), create) {
        let mut task = task.borrow_mut();
        let fd = task.alloc_fd();
        task.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn close(fd: usize) -> isize {
    let task = current_task();
    let mut task = task.borrow_mut();
    let fd_table = &task.fd_table;
    if fd >= fd_table.len() {
        return -1;
    }
    if fd_table[fd].is_none() {
        return -1;
    }
    task.fd_table[fd].take();
    0
}

use crate::fs::make_pipe;

pub fn pipe(pipe_fd: *mut usize) -> isize {
    let task = current_task();
    let mut task = task.borrow_mut();

    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = task.alloc_fd();
    task.fd_table[read_fd] = Some(pipe_read);
    let write_fd = task.alloc_fd();
    task.fd_table[write_fd] = Some(pipe_write);
    let pipe_fd = task.memory_set.page_table.translate_type::<[usize; 2]>(pipe_fd as usize);
    pipe_fd[0] = read_fd;
    pipe_fd[1] = write_fd;
    0
}
