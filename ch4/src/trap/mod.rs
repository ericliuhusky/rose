use crate::syscall::系统调用;
use core::arch::global_asm;
mod context;
pub use context::Context;
use riscv_register::{scause::{self, Exception, Interrupt}, stvec};
use crate::timer::为下一次时钟中断定时;
use crate::task::任务管理器;

global_asm!(include_str!("restore.s"));
global_asm!(include_str!("save.s"));

pub fn 初始化() {
    extern "C" {
        fn __trap_entry();
    }
    // 设置异常处理入口地址为__trap_entry
    stvec::write(__trap_entry as usize);
}


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn trap_handler() {
    let 当前任务的地址空间 = &任务管理器::当前任务().地址空间;
    let 上下文 = 当前任务的地址空间.陷入上下文();
    match scause::read() {
        Exception::UserEnvCall => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.sepc += 4;
            上下文.x[10] = 系统调用(
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
            任务管理器::终止并运行下一个任务();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            任务管理器::终止并运行下一个任务();
        }
        Exception::Interrupt(Interrupt::Timer) => {
            为下一次时钟中断定时();
            任务管理器::暂停并运行下一个任务();
        }
        _ => {
            
        }
    }
    let user_satp = 当前任务的地址空间.token();
    extern "C" {
        fn __restore(user_satp: usize);
    }
    unsafe {
        __restore(user_satp);
    }
}
