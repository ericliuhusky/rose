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
    fn exit() -> isize {
        exit()
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
    fn waitpid(pid: usize) -> isize {
        waitpid(pid)
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
    fn mutex_create() -> isize {
        mutex_create()
    }
    fn mutex_lock(mutex_id: usize) -> isize {
        mutex_lock(mutex_id)
    }
    fn mutex_unlock(mutex_id: usize) -> isize {
        mutex_unlock(mutex_id)
    }
    fn semaphore_create(res_count: usize) -> isize {
        semaphore_create(res_count)
    }
    fn semaphore_down(sem_id: usize) -> isize {
        semaphore_down(sem_id)
    }
    fn semaphore_up(sem_id: usize) -> isize {
        semaphore_up(sem_id)
    }
    fn connect(fd: usize, ip: u32, port: u16) -> isize {
        connect(fd, ip, port)
    }
    fn listen(fd: usize) -> isize {
        listen(fd)
    }
    fn accept(fd: usize) -> isize {
        accept(fd)
    }
    fn socket(tcp: bool) -> isize {
        socket(tcp)
    }
    fn bind(fd: usize, port: u16) -> isize {
        bind(fd, port)
    }
}

mod 系统调用_输出 {
    use crate::task::current_process;

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        let process = current_process();
        let fd_table = &process.fd_table;
        if let Some(file) = fd_table.get(fd) {
            let mut file = file.clone();
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
    use crate::task::exit_and_run_next;

    pub fn exit() -> isize {
        println!("[kernel] Application exited");
        exit_and_run_next();
        -1
    }
}

mod 系统调用_读取 {
    use crate::task::current_process;

    pub fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        let process = current_process();
        let fd_table = &process.fd_table;
        if let Some(file) = fd_table.get(fd) {
            let mut file = file.clone();
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
    use crate::task::suspend_and_run_next;

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
    use crate::task::{current_process, PROCESSES};

    pub fn getpid() -> isize {
        current_process().pid.unwrap() as isize
    }

    pub fn fork() -> isize {
        let mut process = current_process();
        let new_process = process.fork();
        let mut task = new_process.main_task();
        task.cx.x[10] = 0;
        let new_pid = new_process.pid.unwrap();
        new_pid as isize
    }

    use crate::fs::open_file;

    pub fn exec(path: *const u8, len: usize) -> isize {
        let mut process = current_process();
        let 应用名称 = process
            .memory_set
            .page_table
            .translate_buffer(path as usize, len)
            .to_string();
        if let Some(elf_inode) = open_file(&应用名称, false) {
            let elf_data = elf_inode.read_all();
            process.exec(&elf_data);
            0
        } else {
            -1
        }
    }

    pub fn waitpid(pid: usize) -> isize {
        if let Some(waited_process) = unsafe { &PROCESSES }.get(pid) {
            if waited_process.is_exited {
                unsafe { &mut PROCESSES }.remove(pid);
                0
            } else {
                -2
            }
        } else {
            -1
        }
    }
}

use crate::fs::{open_file, File};
use crate::mutex::Mutex;
use crate::net::tcp::TCP;
use crate::semaphore::Semaphore;
use mutrc::MutRc;
use crate::task::{current_task, current_process, add_task};
use crate::task::task::Task;

pub fn open(path: *const u8, len: usize, create: u32) -> isize {
    let mut process = current_process();
    let path = process
        .memory_set
        .page_table
        .translate_buffer(path as usize, len)
        .to_string();
    let create = create != 0;
    if let Some(inode) = open_file(path.as_str(), create) {
        let fd = process.fd_table.insert(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn close(fd: usize) -> isize {
    let mut process = current_process();
    let fd_table = &process.fd_table;
    process.fd_table.remove(fd);
    0
}

use crate::fs::Pipe;

pub fn pipe(pipe_fd: *mut usize) -> isize {
    let mut process = current_process();

    let (pipe_read, pipe_write) = Pipe::new_pair();
    let read_fd = process.fd_table.insert(pipe_read);
    let write_fd = process.fd_table.insert(pipe_write);
    let pipe_fd = process.memory_set.page_table.translate_type::<[usize; 2]>(pipe_fd as usize);
    pipe_fd[0] = read_fd;
    pipe_fd[1] = write_fd;
    0
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
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
    new_task_tid as isize
}

pub fn waittid(tid: usize) -> isize {
    let task = current_task();
    let process = task.process.upgrade().unwrap();

    let waited_task = process.tasks.get(tid);
    if let Some(waited_task) = waited_task {
        if waited_task.is_exited {
            0
        } else {
            -2
        }
    } else {
        -1
    }
}

fn mutex_create() -> isize {
    let mut process = current_process();
    let mutex = MutRc::new(Mutex::new());
    let id = process.mutexs.insert(mutex);
    id as isize
}

fn mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let mut mutex = process.mutexs.get(mutex_id).unwrap().clone();
    mutex.lock();
    0
}

fn mutex_unlock(mutex_id: usize) -> isize {
    let process = current_process();
    let mut mutex = process.mutexs.get(mutex_id).unwrap().clone();
    mutex.unlock();
    0
}

fn semaphore_create(res_count: usize) -> isize {
    let mut process = current_process();
    let semaphore = MutRc::new(Semaphore::new(res_count));
    let id = process.semaphores.insert(semaphore);
    id as isize
}

fn semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let mut semaphore = process.semaphores.get(sem_id).unwrap().clone();
    semaphore.down();
    0
}

fn semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let mut semaphore = process.semaphores.get(sem_id).unwrap().clone();
    semaphore.up();
    0
}



use crate::net::port_table;
use crate::net::udp::UDP;
use crate::net::{IPv4, net_arp, busy_wait_accept};

// just support udp
fn connect(fd: usize, ip: u32, port: u16) -> isize {
    unimplemented!();
}

// listen a port
fn listen(fd: usize) -> isize {
    let mut process = current_process();
    let mut socket = process.fd_table.get(fd).unwrap().clone();
    let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<TCP>) };
    port_table::listen(socket.source_port);
    0
}

// accept a tcp connection
fn accept(fd: usize) -> isize {
    let mut process = current_process();
    let mut socket = process.fd_table.get(fd).unwrap().clone();
    let socket =  unsafe { &mut *(&mut socket as *mut _ as *mut MutRc<TCP>) };

    net_arp();
    let tcp_socket = busy_wait_accept(socket.source_port);

    let fd = process.fd_table.insert(MutRc::new(tcp_socket));

    fd as isize
}

fn socket(tcp: bool) -> isize {
    let mut process = current_process();
    let socket: MutRc<dyn File> = if tcp {
        MutRc::new(TCP::new_server())
    } else {
        MutRc::new(UDP::new())
    };
    let fd = process.fd_table.insert(socket);
    fd as isize
}

fn bind(fd: usize, port: u16) -> isize {
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
