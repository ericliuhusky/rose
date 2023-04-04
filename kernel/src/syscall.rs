use exception::context::Context;
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
    fn thread_create(entry: usize, arg: usize) -> isize {
        thread_create(entry, arg)
    }
    fn waittid(tid: usize) -> isize {
        waittid(tid)
    }
}

mod 系统调用_输出 {
    use crate::task::{TaskManager, current_task, current_process};
    use page_table::VA;

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        let process = current_process();
        let fd_table = &process.fd_table;
        if fd >= fd_table.len() {
            return -1;
        }
        if let Some(file) = &fd_table[fd] {
            let file = file.clone();
            let buf = process
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
    use crate::task::{TaskManager, current_task, current_process};
    use page_table::VA;

    pub fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        let process = current_process();
        let fd_table = &process.fd_table;
        if fd >= fd_table.len() {
            return -1;
        }
        if let Some(file) = &fd_table[fd] {
            let file = file.clone();
            let buf = process
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
    use crate::task::{TaskManager, current_task, add_task, current_process};
    use alloc::string::String;

    pub fn getpid() -> isize {
        current_process().pid.0 as isize
    }

    pub fn fork() -> isize {
        let mut process = current_process();
        let new_process = process.fork();
        let task = new_process.main_task();
        let mut task = task.borrow_mut();
        task.cx.x[10] = 0;
        let new_pid = new_process.pid.0;
        new_pid as isize
    }

    use crate::fs::open_file;

    pub fn exec(path: *const u8, len: usize) -> isize {
        let mut process = current_process();
        let 应用名称 = process
            .memory_set
            .read_str(path as usize, len);
        if let Some(elf_inode) = open_file(&应用名称, false) {
            let elf_data = elf_inode.read_all();
            process.exec(&elf_data);
            0
        } else {
            -1
        }
    }

    pub fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        let mut process = current_process();
        if !process
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.pid.0)
        {
            return -1;
        }

        let pair = process.children.iter().enumerate().find(|(_, p)| {
            p.is_exited && (pid == -1 || pid as usize == p.pid.0)
        });
        if let Some((idx, _)) = pair {
            let child = process.children.remove(idx);
            let found_pid = child.pid.0;
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
use crate::task::{current_task, current_process, add_task};
use crate::task::{task::Task, TaskManager};
use alloc::string::String;
use page_table::VA;
use alloc::rc::Rc;
use core::cell::RefCell;

pub fn open(path: *const u8, len: usize, create: u32) -> isize {
    let mut process = current_process();
    let path = process
        .memory_set
        .read_str(path as usize, len);
    let create = create != 0;
    if let Some(inode) = open_file(path.as_str(), create) {
        let fd = process.alloc_fd();
        process.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn close(fd: usize) -> isize {
    let mut process = current_process();
    let fd_table = &process.fd_table;
    if fd >= fd_table.len() {
        return -1;
    }
    if fd_table[fd].is_none() {
        return -1;
    }
    process.fd_table[fd].take();
    0
}

use crate::fs::make_pipe;

pub fn pipe(pipe_fd: *mut usize) -> isize {
    let mut process = current_process();

    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = process.alloc_fd();
    process.fd_table[read_fd] = Some(pipe_read);
    let write_fd = process.alloc_fd();
    process.fd_table[write_fd] = Some(pipe_write);
    let pipe_fd = process.memory_set.page_table.translate_type::<[usize; 2]>(pipe_fd as usize);
    pipe_fd[0] = read_fd;
    pipe_fd[1] = write_fd;
    0
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
    let task = current_task();
    let mut process = task.borrow_mut().process.upgrade().unwrap();
    let new_task = Rc::new(RefCell::new(Task::new(
        process.clone(),
    )));
    add_task(Rc::clone(&new_task));
    let mut new_task_inner = new_task.borrow_mut();
    let new_task_tid = new_task_inner.tid;
    process.tasks.insert(new_task_tid, Rc::clone(&new_task));
    let ustack_top = new_task_inner.user_stack_top();
    new_task_inner.cx = Context::app_init(
        entry,
        ustack_top,
    );
    new_task_inner.cx.x[10] = arg;
    new_task_tid as isize
}

pub fn waittid(tid: usize) -> isize {
    let task = current_task();
    let process = task.borrow_mut().process.upgrade().unwrap();

    let waited_task = process.tasks.get(&tid);
    if let Some(waited_task) = waited_task {
        if waited_task.borrow_mut().is_exited {
            0
        } else {
            -2
        }
    } else {
        -1
    }
}
