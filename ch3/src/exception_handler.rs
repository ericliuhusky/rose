use crate::task;
use crate::syscall::SysFuncImpl;
use core::arch::global_asm;
use riscv_register::{scause::{self, Exception, Interrupt}, stvec};
use crate::timer::为下一次时钟中断定时;
use exception::context::Context;
use crate::segment::CONTEXT_START_ADDR;
use sys_func::sys_func;


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn exception_handler() {
    let 上下文 = unsafe { &mut *(CONTEXT_START_ADDR as *mut Context) };
    match scause::read() {
        Exception::UserEnvCall => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.sepc += 4;
            let result =
                sys_func::<SysFuncImpl>(上下文.x[17], [上下文.x[10], 上下文.x[11], 上下文.x[12]]);
            match result {
                Ok(ret) => 上下文.x[10] = ret as usize,
                Err(id) => {
                    println!("[kernel] Unsupported syscall_id: {}", id);
                }
            }
        }
        Exception::StoreFault | Exception::StorePageFault => {
            println!("[kernel] PageFault in application, kernel killed it.");
            task::exit_and_run_next();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            task::exit_and_run_next();
        }
        Exception::Interrupt(Interrupt::Timer) => {
            为下一次时钟中断定时();
            task::suspend_and_run_next();
        }
        _ => {
            
        }
    }
}
