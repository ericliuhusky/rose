use core::arch::asm;

#[inline(always)]
fn sys_call(id: usize, args: [usize; 3]) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

use sys_call_id::*;

pub fn read(fd: usize, buffer: &mut [u8]) -> usize {
    sys_call(SYS_READ, [fd, buffer as *mut [u8] as *mut u8 as usize, buffer.len()])
}
pub fn write(fd: usize, buf: &[u8]) -> usize {
    sys_call(SYS_WRITE, [fd, buf as *const [u8] as *const u8 as usize, buf.len()])
}
pub fn open(path: &str, create: usize) -> usize {
    sys_call(SYS_OPEN, [path.as_ptr() as usize, path.len(), create])
}
pub fn close(fd: usize) -> usize {
    sys_call(SYS_CLOSE, [fd, 0, 0])
}

pub fn putchar(c: usize) {
    sys_call(SYS_PUTCHAR, [c, 0, 0]);
}

pub fn getchar() -> usize {
    sys_call(SYS_GETCHAR, [0, 0, 0])
}

pub fn exit() -> ! {
    sys_call(SYS_EXIT, [0, 0, 0]);
    panic!("exit")
}

pub fn yield_() -> usize {
    sys_call(SYS_YIELD, [0, 0, 0])
}

pub fn get_time() -> usize {
    sys_call(SYS_GET_TIME, [0, 0, 0])
}

pub fn getpid() -> usize {
    sys_call(SYS_GETPID, [0, 0, 0])
}

pub fn fork() -> usize {
    sys_call(SYS_FORK, [0, 0, 0])
}

pub fn exec(path: &str) -> usize {
    sys_call(SYS_EXEC, [path.as_ptr() as usize, path.len(), 0])
}

pub fn waitpid(pid: usize) -> usize {
    sys_call(SYS_WAITPID, [pid, 0, 0])
}

pub fn pipe(pipe: &mut [usize]) -> usize {
    sys_call(SYS_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}

pub fn thread_create(entry: usize, arg: usize) -> usize {
    sys_call(SYS_THREAD_CREATE, [entry, arg, 0])
}

pub fn waittid(tid: usize) -> usize {
    sys_call(SYS_WAITTID, [tid, 0, 0])
}

pub fn mutex_create() -> usize {
    sys_call(SYS_MUTEX_CREATE, [0, 0, 0])
}

pub fn mutex_lock(mutex_id: usize) -> usize {
    sys_call(SYS_MUTEX_LOCK, [mutex_id, 0, 0])
}

pub fn mutex_unlock(mutex_id: usize) -> usize {
    sys_call(SYS_MUTEX_UNLOCK, [mutex_id, 0, 0])
}

pub fn semaphore_create(res_count: usize) -> usize {
    sys_call(SYS_SEMAPHORE_CREATE, [res_count, 0, 0])
}

pub fn semaphore_down(sem_id: usize) -> usize {
    sys_call(SYS_SEMAPHORE_DOWN, [sem_id, 0, 0])
}

pub fn semaphore_up(sem_id: usize) -> usize {
    sys_call(SYS_SEMAPHORE_UP, [sem_id, 0, 0])
}

pub fn connect(fd: usize, ip: u32, port: u16) -> usize {
    sys_call(SYS_CONNECT, [fd, ip as usize, port as usize])
}

pub fn listen(fd: usize) -> usize {
    sys_call(SYS_LISTEN, [fd as usize, 0, 0])
}

pub fn accept(fd: usize) -> usize {
    sys_call(SYS_ACCEPT, [fd, 0, 0])
}

pub fn socket(tcp: bool) -> usize {
    sys_call(SYS_SOCKET, [tcp as usize, 0, 0])
}

pub fn bind(fd: usize, port: u16) -> usize {
    sys_call(SYS_BIND, [fd, port as usize, 0])
}
