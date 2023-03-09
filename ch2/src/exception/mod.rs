mod context;
pub mod restore;
mod save;
use crate::{batch::应用管理器, segment::CONTEXT_START_ADDR};
use crate::syscall::sys_func;
pub use context::Context;
use riscv_register::{scause::{self, Exception}, stvec};
pub use restore::restore_context;

pub fn 初始化() {
    extern "C" {
        fn __save();
    }    
    // 设置异常处理入口地址为__save
    stvec::write(__save as usize);
}

#[no_mangle]
/// 处理中断、异常或系统调用
pub fn exception_handler() {
    let 上下文 = unsafe { &mut *(CONTEXT_START_ADDR as *mut Context) };
    match scause::read() {
        Exception::UserEnvCall => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.sepc += 4;
            上下文.x[10] = sys_func(
                上下文.x[17],
                [
                    上下文.x[10],
                    上下文.x[11], 
                    上下文.x[12]
                ]
            ) as usize;
        }
        Exception::StoreFault | Exception::StorePageFault => {
            println!("[kernel] PageFault in application, kernel killed it.");
            应用管理器::运行下一个应用();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            应用管理器::运行下一个应用();
        }
        _ => {
            
        }
    }
}
