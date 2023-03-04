mod context;
use crate::batch::应用管理器;
use crate::syscall::系统调用;
use core::arch::global_asm;
pub use context::陷入上下文;
use riscv_register::{scause::{self, Exception}, stvec};

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
pub fn trap_handler(上下文: &mut 陷入上下文) -> &mut 陷入上下文 {
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
            应用管理器::运行下一个应用();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            应用管理器::运行下一个应用();
        }
        _ => {
            
        }
    }
    上下文
}
