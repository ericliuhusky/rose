#![no_std]

use sys_call_id::*;

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
        SYS_PIPE => Ok(SysFuncImpl::pipe(args[0] as *mut usize)),
        _ => Err(id),
    }
}

pub trait SysFunc {
    fn read(fd: usize, buf: *const u8, len: usize) -> isize {
        panic!()
    }
    fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        panic!()
    }
    fn exit(exit_code: i32) -> isize {
        panic!()
    }
    fn yield_() -> isize {
        panic!()
    }
    fn get_time() -> isize {
        panic!()
    }
    fn getpid() -> isize {
        panic!()
    }
    fn fork() -> isize {
        panic!()
    }
    fn exec(path: *const u8, len: usize) -> isize {
        panic!()
    }
    fn waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        panic!()
    }
    fn putchar(c: usize) -> isize {
        panic!()
    }
    fn getchar() -> isize {
        panic!()
    }
    fn open(path: *const u8, len: usize, create: u32) -> isize {
        panic!()
    }
    fn close(fd: usize) -> isize {
        panic!()
    }
    fn pipe(pipe_fd: *mut usize) -> isize {
        panic!()
    }
}
