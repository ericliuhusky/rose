#![no_std]

const SYS_READ: usize = 0;
const SYS_WRITE: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_YIELD: usize = 3;
const SYS_GET_TIME: usize = 4;
const SYS_GETPID: usize = 5;
const SYS_FORK: usize = 6;
const SYS_EXEC: usize = 7;
const SYS_WAITPID: usize = 8;
const SYS_PUTCHAR: usize = 9;
const SYS_GETCHAR: usize = 10;
const SYS_OPEN: usize = 11;
const SYS_CLOSE: usize = 12;

pub fn sys_func<SysFuncImpl: SysFunc>(id: usize, args: [usize; 3]) -> Result<isize, usize> {
    match id {
        SYS_READ => Ok(SysFuncImpl::read(args[0], args[1] as *const u8, args[2])),
        SYS_WRITE => Ok(SysFuncImpl::write(args[0], args[1] as *const u8, args[2])),
        SYS_EXIT => Ok(SysFuncImpl::exit(args[0] as i32)),
        SYS_YIELD => Ok(SysFuncImpl::yield_()),
        SYS_GET_TIME => Ok(SysFuncImpl::get_time()),
        SYS_GETPID => Ok(SysFuncImpl::getpid()),
        SYS_FORK => Ok(SysFuncImpl::fork()),
        SYS_EXEC => Ok(SysFuncImpl::exec(args[0] as *const u8, args[1])),
        SYS_WAITPID => Ok(SysFuncImpl::waitpid(args[0] as isize, args[1] as *mut i32)),
        SYS_PUTCHAR => Ok(SysFuncImpl::putchar(args[0])),
        SYS_GETCHAR => Ok(SysFuncImpl::getchar()),
        SYS_OPEN => Ok(SysFuncImpl::open(
            args[0] as *const u8,
            args[1],
            args[2] as u32,
        )),
        SYS_CLOSE => Ok(SysFuncImpl::close(args[0])),
        _ => Err(id),
    }
}

pub trait SysFunc {
    fn read(fd: usize, buf: *const u8, len: usize) -> isize;
    fn write(fd: usize, buf: *const u8, len: usize) -> isize;
    fn exit(exit_code: i32) -> isize;
    fn yield_() -> isize;
    fn get_time() -> isize;
    fn getpid() -> isize;
    fn fork() -> isize;
    fn exec(path: *const u8, len: usize) -> isize;
    fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize;
    fn putchar(c: usize) -> isize;
    fn getchar() -> isize;
    fn open(path: *const u8, len: usize, create: u32) -> isize;
    fn close(fd: usize) -> isize;
}
