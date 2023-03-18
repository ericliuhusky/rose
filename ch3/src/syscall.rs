use crate::task;
use sys_func::SysFunc;

pub struct SysFuncImpl;

impl SysFunc for SysFuncImpl {
    fn exit(exit_code: i32) -> isize {
        println!("[kernel] Application exited with code {}", exit_code);
        task::exit_and_run_next();
        -1
    }
    fn putchar(c: usize) -> isize {
        sbi_call::putchar(c);
        c as isize
    }
}
