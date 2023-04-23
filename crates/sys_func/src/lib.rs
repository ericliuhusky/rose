#![no_std]

use sys_call_id::*;

pub fn sys_func<SysFuncImpl: SysFunc>(id: usize, args: [usize; 3]) -> Result<usize, usize> {
    match id {
        SYS_READ => Ok(SysFuncImpl::read(args[0], args[1] as *const u8, args[2])),
        SYS_WRITE => Ok(SysFuncImpl::write(args[0], args[1] as *const u8, args[2])),
        SYS_EXIT => Ok(SysFuncImpl::exit()),
        SYS_YIELD => Ok(SysFuncImpl::yield_()),
        SYS_GET_TIME => Ok(SysFuncImpl::get_time()),
        SYS_GETPID => Ok(SysFuncImpl::getpid()),
        SYS_FORK => Ok(SysFuncImpl::fork()),
        SYS_EXEC => Ok(SysFuncImpl::exec(args[0] as *const u8, args[1])),
        SYS_WAITPID => Ok(SysFuncImpl::waitpid(args[0])),
        SYS_PUTCHAR => Ok(SysFuncImpl::putchar(args[0])),
        SYS_GETCHAR => Ok(SysFuncImpl::getchar()),
        SYS_OPEN => Ok(SysFuncImpl::open(
            args[0] as *const u8,
            args[1],
            args[2] as u32,
        )),
        SYS_CLOSE => Ok(SysFuncImpl::close(args[0])),
        SYS_PIPE => Ok(SysFuncImpl::pipe(args[0] as *mut usize)),
        SYS_THREAD_CREATE => Ok(SysFuncImpl::thread_create(args[0], args[1])),
        SYS_WAITTID => Ok(SysFuncImpl::waittid(args[0])),
        SYS_MUTEX_CREATE => Ok(SysFuncImpl::mutex_create()),
        SYS_MUTEX_LOCK => Ok(SysFuncImpl::mutex_lock(args[0])),
        SYS_MUTEX_UNLOCK => Ok(SysFuncImpl::mutex_unlock(args[0])),
        SYS_SEMAPHORE_CREATE => Ok(SysFuncImpl::semaphore_create(args[0])),
        SYS_SEMAPHORE_DOWN => Ok(SysFuncImpl::semaphore_down(args[0])),
        SYS_SEMAPHORE_UP => Ok(SysFuncImpl::semaphore_up(args[0])),
        SYS_CONNECT => Ok(SysFuncImpl::connect(args[0], args[1] as u32, args[2] as u16)),
        SYS_LISTEN => Ok(SysFuncImpl::listen(args[0])),
        SYS_ACCEPT => Ok(SysFuncImpl::accept(args[0])),
        SYS_SOCKET => Ok(SysFuncImpl::socket(args[0] == 1)),
        SYS_BIND => Ok(SysFuncImpl::bind(args[0], args[1] as u16)),
        _ => Err(id),
    }
}

pub trait SysFunc {
    fn read(fd: usize, buf: *const u8, len: usize) -> usize;
    fn write(fd: usize, buf: *const u8, len: usize) -> usize;
    fn exit() -> usize;
    fn yield_() -> usize;
    fn get_time() -> usize;
    fn getpid() -> usize;
    fn fork() -> usize;
    fn exec(path: *const u8, len: usize) -> usize;
    fn waitpid(pid: usize) -> usize;
    fn putchar(c: usize) -> usize;
    fn getchar() -> usize;
    fn open(path: *const u8, len: usize, create: u32) -> usize;
    fn close(fd: usize) -> usize;
    fn pipe(pipe_fd: *mut usize) -> usize;
    fn thread_create(entry: usize, arg: usize) -> usize;
    fn waittid(tid: usize) -> usize;
    fn mutex_create() -> usize;
    fn mutex_lock(mutex_id: usize) -> usize;
    fn mutex_unlock(mutex_id: usize) -> usize;
    fn semaphore_create(res_count: usize) -> usize;
    fn semaphore_down(sem_id: usize) -> usize;
    fn semaphore_up(sem_id: usize) -> usize;
    fn connect(fd: usize, ip: u32, port: u16) -> usize;
    fn listen(fd: usize) -> usize;
    fn accept(fd: usize) -> usize;
    fn socket(tcp: bool) -> usize;
    fn bind(fd: usize, port: u16) -> usize;
}
