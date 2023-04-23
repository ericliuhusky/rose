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
    fn read(fd: usize, buf: usize, len: usize) -> usize {
        read(fd, buf, len)
    }
    fn write(fd: usize, buf: usize, len: usize) -> usize {
        write(fd, buf, len)
    }
    fn exit() -> usize {
        exit()
    }
    fn yield_() -> usize {
        yield_()
    }
    fn get_time() -> usize {
        get_time()
    }
    fn getpid() -> usize {
        getpid()
    }
    fn fork() -> usize {
        fork()
    }
    fn exec(path: usize, len: usize) -> usize {
        exec(path, len)
    }
    fn waitpid(pid: usize) -> usize {
        waitpid(pid)
    }
    fn putchar(c: usize) -> usize {
        putchar(c)
    }
    fn getchar() -> usize {
        getchar()
    }
    fn open(path: usize, len: usize, create: bool) -> usize {
        open(path, len, create)
    }
    fn close(fd: usize) -> usize {
        close(fd)
    }
    fn pipe(pipe_fd: usize) -> usize {
        pipe(pipe_fd)
    }
    fn thread_create(entry: usize, arg: usize) -> usize {
        thread_create(entry, arg)
    }
    fn waittid(tid: usize) -> usize {
        waittid(tid)
    }
    fn mutex_create() -> usize {
        mutex_create()
    }
    fn mutex_lock(mutex_id: usize) -> usize {
        mutex_lock(mutex_id)
    }
    fn mutex_unlock(mutex_id: usize) -> usize {
        mutex_unlock(mutex_id)
    }
    fn semaphore_create(res_count: usize) -> usize {
        semaphore_create(res_count)
    }
    fn semaphore_down(sem_id: usize) -> usize {
        semaphore_down(sem_id)
    }
    fn semaphore_up(sem_id: usize) -> usize {
        semaphore_up(sem_id)
    }
    fn connect(fd: usize, ip: u32, port: u16) -> usize {
        connect(fd, ip, port)
    }
    fn listen(fd: usize) -> usize {
        listen(fd)
    }
    fn accept(fd: usize) -> usize {
        accept(fd)
    }
    fn socket(tcp: bool) -> usize {
        socket(tcp)
    }
    fn bind(fd: usize, port: u16) -> usize {
        bind(fd, port)
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
        if let Some(elf_inode) = open_file(&应用名称, false) {
            let elf_data = elf_inode.read_all();
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

use crate::fs::{open_file, File};
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
    if let Some(inode) = open_file(path.as_str(), create) {
        let fd = process.fd_table.insert(inode);
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
use crate::net::{IPv4, net_arp, busy_wait_accept};

// just support udp
fn connect(fd: usize, ip: u32, port: u16) -> usize {
    unimplemented!();
}

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
    let socket: MutRc<dyn File> = if tcp {
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
