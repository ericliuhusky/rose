use crate::syscall::系统调用;
use core::arch::global_asm;
mod context;
mod scause;
pub use context::陷入上下文;
use scause::{Trap, Exception, Interrupt};
use crate::timer::为下一次时钟中断定时;
use crate::task::任务管理器;
use crate::格式化输出并换行;

global_asm!(include_str!("trap.s"));

pub fn 初始化() {
    extern "C" {
        fn __trap_entry();
    }
    // 设置异常处理入口地址为__trap_entry
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) __trap_entry as usize);
    }
}


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn trap_handler() {
    let 上下文 = 任务管理器::当前页表().translated_trap_context();
    match scause::read().cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.触发异常指令地址 += 4;
            上下文.通用寄存器[10] = 系统调用(
                上下文.通用寄存器[17],
                [
                    上下文.通用寄存器[10],
                    上下文.通用寄存器[11], 
                    上下文.通用寄存器[12]
                ]
            ) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            格式化输出并换行!("[kernel] PageFault in application, kernel killed it.");
            任务管理器::终止并运行下一个任务();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            格式化输出并换行!("[kernel] IllegalInstruction in application, kernel killed it.");
            任务管理器::终止并运行下一个任务();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            为下一次时钟中断定时();
            任务管理器::暂停并运行下一个任务();
        }
        _ => {
            
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() {
    let user_satp = 任务管理器::当前页表().token();
    extern "C" {
        fn __restore(user_satp: usize);
    }
    unsafe {
        __restore(user_satp);
    }
}
