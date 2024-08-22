use exception::context::Context;
use 系统调用_时钟计数器::get_time;
use 系统调用_终止::exit;
use 系统调用_让出时间片::yield_;
use 系统调用_读取::{getchar, read};
use 系统调用_输出::{putchar, write};
use 系统调用_进程::{exec, fork, getpid, waitpid};


use sys_call_id::*;

pub fn syscall(id: usize, args: [usize; 3]) -> Result<usize, usize> {
    match id {
        SYS_READ => Ok(read(args[0], args[1], args[2])),
        SYS_WRITE => Ok(write(args[0], args[1], args[2])),
        SYS_EXIT => Ok(exit()),
        SYS_YIELD => Ok(yield_()),
        SYS_GET_TIME => Ok(get_time()),
        SYS_GETPID => Ok(getpid()),
        SYS_FORK => Ok(fork()),
        SYS_EXEC => Ok(exec(args[0], args[1])),
        SYS_WAITPID => Ok(waitpid(args[0])),
        SYS_PUTCHAR => Ok(putchar(args[0])),
        SYS_GETCHAR => Ok(getchar()),
        SYS_OPEN => Ok(open(
            args[0],
            args[1],
            args[2] == 1,
        )),
        SYS_CLOSE => Ok(close(args[0])),
        SYS_PIPE => Ok(pipe(args[0])),
        SYS_THREAD_CREATE => Ok(thread_create(args[0], args[1])),
        SYS_WAITTID => Ok(waittid(args[0])),
        SYS_MUTEX_CREATE => Ok(mutex_create()),
        SYS_MUTEX_LOCK => Ok(mutex_lock(args[0])),
        SYS_MUTEX_UNLOCK => Ok(mutex_unlock(args[0])),
        SYS_SEMAPHORE_CREATE => Ok(semaphore_create(args[0])),
        SYS_SEMAPHORE_DOWN => Ok(semaphore_down(args[0])),
        SYS_SEMAPHORE_UP => Ok(semaphore_up(args[0])),
        SYS_LISTEN => Ok(listen(args[0])),
        SYS_ACCEPT => Ok(accept(args[0])),
        SYS_SOCKET => Ok(socket(args[0] == 1)),
        SYS_BIND => Ok(bind(args[0], args[1] as u16)),
        _ => Err(id),
    }
}

mod 系统调用_输出 {
    use crate::task::current_process;

    pub fn write(fd: usize, buf: usize, len: usize) -> usize {
        let process = current_process();
        let fd_table = &process.fd_table;
        let mut file = fd_table.get(fd).unwrap().clone();
        let buf = process
            .memory_set
            .page_table
            .translate_buffer(buf, len);
        file.write(buf)
    }

    pub fn putchar(c: usize) -> usize {
        sbi_call::putchar(c);
        c
    }
}

mod 系统调用_终止 {
    use crate::task::exit_and_run_next;

    pub fn exit() -> usize {
        println!("[kernel] Application exited");
        exit_and_run_next();
        unreachable!()
    }
}

mod 系统调用_读取 {
    use crate::task::current_process;

    pub fn read(fd: usize, buf: usize, len: usize) -> usize {
        let process = current_process();
        let fd_table = &process.fd_table;
        let mut file = fd_table.get(fd).unwrap().clone();
        let buf = process
            .memory_set
            .page_table
            .translate_buffer(buf, len);
        file.read(buf)
    }

    pub fn getchar() -> usize {
        sbi_call::getchar()
    }
}

mod 系统调用_让出时间片 {
    use crate::task::suspend_and_run_next;

    pub fn yield_() -> usize {
        suspend_and_run_next();
        0
    }
}

mod 系统调用_时钟计数器 {
    use crate::timer::读取时钟计数器的毫秒值;

    pub fn get_time() -> usize {
        读取时钟计数器的毫秒值()
    }
}

mod 系统调用_进程 {
    use crate::task::{current_process, PROCESSES};

    pub fn getpid() -> usize {
        current_process().pid.unwrap()
    }

    pub fn fork() -> usize {
        let mut process = current_process();
        let new_process = process.fork();
        let mut task = new_process.main_task();
        task.cx.x[10] = 0;
        let new_pid = new_process.pid.unwrap();
        new_pid
    }

    use crate::fs::open_file;

    pub fn exec(path: usize, len: usize) -> usize {
        let mut process = current_process();
        let 应用名称 = process
            .memory_set
            .page_table
            .translate_buffer(path, len)
            .to_string();
        if let Some(elf_f) = open_file(&应用名称, false) {
            let elf_data = elf_f.read_all();
            process.exec(&elf_data);
            1
        } else {
            0
        }
    }

    pub fn waitpid(pid: usize) -> usize {
        let waited_process = unsafe { &PROCESSES }.get(pid).unwrap();
        if waited_process.is_exited {
            unsafe { &mut PROCESSES }.remove(pid);
            1
        } else {
            0
        }
    }
}

use crate::fs::{open_file, FileInterface};
use crate::mutex::Mutex;
use crate::net::tcp::TCP;
use crate::semaphore::Semaphore;
use alloc_ext::rc::MutRc;
use crate::task::{current_task, current_process, add_task};
use crate::task::task::Task;

pub fn open(path: usize, len: usize, create: bool) -> usize {
    let mut process = current_process();
    let path = process
        .memory_set
        .page_table
        .translate_buffer(path, len)
        .to_string();
    if let Some(f) = open_file(path.as_str(), create) {
        let fd = process.fd_table.insert(f);
        fd
    } else {
        0
    }
}

pub fn close(fd: usize) -> usize {
    let mut process = current_process();
    let fd_table = &process.fd_table;
    process.fd_table.remove(fd);
    0
}

use crate::fs::Pipe;

pub fn pipe(pipe_fd: usize) -> usize {
    let mut process = current_process();

    let (pipe_read, pipe_write) = Pipe::new_pair();
    let read_fd = process.fd_table.insert(pipe_read);
    let write_fd = process.fd_table.insert(pipe_write);
    let pipe_fd = process.memory_set.page_table.translate_type::<[usize; 2]>(pipe_fd);
    pipe_fd[0] = read_fd;
    pipe_fd[1] = write_fd;
    0
}

pub fn thread_create(entry: usize, arg: usize) -> usize {
    let task = current_task();
    let mut process = task.process.upgrade().unwrap();
    let mut new_task = MutRc::new(Task::new(
        process.clone(),
    ));
    let new_task_tid = process.tasks.insert(new_task.clone());
    new_task.tid = Some(new_task_tid);
    add_task(new_task.clone());
    let ustack_top = new_task.user_stack_top();
    new_task.cx = Context::app_init(
        entry,
        ustack_top,
    );
    new_task.cx.x[10] = arg;
    new_task_tid
}

pub fn waittid(tid: usize) -> usize {
    let task = current_task();
    let process = task.process.upgrade().unwrap();

    let waited_task = process.tasks.get(tid).unwrap();
    if waited_task.is_exited {
        1
    } else {
        0
    }
}

fn mutex_create() -> usize {
    let mut process = current_process();
    let mutex = MutRc::new(Mutex::new());
    let id = process.mutexs.insert(mutex);
    id
}

fn mutex_lock(mutex_id: usize) -> usize {
    let process = current_process();
    let mut mutex = process.mutexs.get(mutex_id).unwrap().clone();
    mutex.lock();
    0
}

fn mutex_unlock(mutex_id: usize) -> usize {
    let process = current_process();
    let mut mutex = process.mutexs.get(mutex_id).unwrap().clone();
    mutex.unlock();
    0
}

fn semaphore_create(res_count: usize) -> usize {
    let mut process = current_process();
    let semaphore = MutRc::new(Semaphore::new(res_count));
    let id = process.semaphores.insert(semaphore);
    id
}

fn semaphore_down(sem_id: usize) -> usize {
    let process = current_process();
    let mut semaphore = process.semaphores.get(sem_id).unwrap().clone();
    semaphore.down();
    0
}

fn semaphore_up(sem_id: usize) -> usize {
    let process = current_process();
    let mut semaphore = process.semaphores.get(sem_id).unwrap().clone();
    semaphore.up();
    0
}



use crate::net::port_table;
use crate::net::udp::UDP;
use crate::net::{net_arp, busy_wait_accept};


// listen a port
fn listen(fd: usize) -> usize {
    let mut process = current_process();
    let mut socket = process.fd_table.get(fd).unwrap().clone();
    let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<TCP>) };
    port_table::listen(socket.source_port);
    0
}

// accept a tcp connection
fn accept(fd: usize) -> usize {
    let mut process = current_process();
    let mut socket = process.fd_table.get(fd).unwrap().clone();
    let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<TCP>) };

    net_arp();
    let tcp_socket = busy_wait_accept(socket.source_port);

    let fd = process.fd_table.insert(MutRc::new(tcp_socket));

    fd
}

fn socket(tcp: bool) -> usize {
    let mut process = current_process();
    let socket: MutRc<dyn FileInterface> = if tcp {
        MutRc::new(TCP::new_server())
    } else {
        MutRc::new(UDP::new())
    };
    let fd = process.fd_table.insert(socket);
    fd
}

fn bind(fd: usize, port: u16) -> usize {
    let process = current_process();
    let mut socket = process.fd_table.get(fd).unwrap().clone();
    match socket.file_type() {
        crate::fs::FileType::TCP => {
            let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<TCP>) };
            socket.source_port = port;
        },
        crate::fs::FileType::UDP => {
            let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<UDP>) };
            socket.source_port = port;
        }
        _ => {}
    }
    0
}
