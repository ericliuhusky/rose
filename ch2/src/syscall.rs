use crate::batch::应用管理器;
use sys_func::SysFunc;

pub struct SysFuncImpl;

impl SysFunc for SysFuncImpl {
    fn exit(exit_code: i32) -> isize {
        println!("[kernel] Application exited with code {}", exit_code);
        应用管理器::recycle();
        应用管理器::运行下一个应用();
        -1
    }
    fn putchar(c: usize) -> isize {
        sbi_call::putchar(c);
        c as isize
    }
}
