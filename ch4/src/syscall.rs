use crate::task::任务管理器;
use sys_func::SysFunc;

pub struct SysFuncImpl;

impl SysFunc for SysFuncImpl {
    fn exit(exit_code: i32) -> isize {
        println!("[kernel] Application exited with code {}", exit_code);
        任务管理器::终止并运行下一个任务();
        -1
    }
    fn putchar(c: usize) -> isize {
        sbi_call::putchar(c);
        c as isize
    }
}
