use crate::syscall::系统调用;
use core::arch::global_asm;
mod context;
pub use context::陷入上下文;
use riscv_register::{scause::{self, Exception, Interrupt}, stvec};
use crate::timer::为下一次时钟中断定时;
use crate::task::任务管理器;

global_asm!(include_str!("trap.s"));

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
    // let 当前任务的地址空间 = 任务管理器::当前任务().borrow().地址空间;
    let 上下文 = 任务管理器::当前任务().地址空间.陷入上下文();
    match scause::read() {
        Exception::UserEnvCall => {
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
        Exception::StoreFault | Exception::StorePageFault => {
            println!("[kernel] PageFault in application, kernel killed it.");
            任务管理器::终止并运行下一个任务(-2);
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            任务管理器::终止并运行下一个任务(-3);
        }
        Exception::Interrupt(Interrupt::Timer) => {
            为下一次时钟中断定时();
            任务管理器::暂停并运行下一个任务();
        }
        _ => {
            
        }
    }
    trap_return();
}

pub fn trap_return() {
    let user_satp = 任务管理器::当前任务().地址空间.token();
    extern "C" {
        fn __restore(user_satp: usize);
    }
    unsafe {
        __restore(user_satp);
    }
}

extern "C" {
    fn __KERNEL_STACK_TOP();
    fn __TRAP_CONTEXT_START();
}

pub fn 内核栈栈顶() -> usize {
    unsafe {
        *(__KERNEL_STACK_TOP as usize as *const usize) 
    }
}

pub fn 应用陷入上下文存放地址() -> usize {
    unsafe {
        *(__TRAP_CONTEXT_START as usize as *const usize) 
    }
}
