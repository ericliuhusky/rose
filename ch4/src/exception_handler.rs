use crate::mm::memory_set::CONTEXT_START_ADDR;
use crate::{syscall::SysFuncImpl, task::current_task};
use crate::task::{TaskManager, exit_and_run_next, suspend_and_run_next};
use crate::timer::为下一次时钟中断定时;
use core::arch::global_asm;
use exception::context::Context;
use exception::restore::restore_context;
use riscv_register::{
    scause::{self, Exception, Interrupt},
    stvec,
};
use sys_func::sys_func;

#[no_mangle]
/// 处理中断、异常或系统调用
pub fn exception_handler() {
    let 当前任务的地址空间 = &current_task().memory_set;
    let cx = 当前任务的地址空间.陷入上下文();
    match scause::read() {
        Exception::UserEnvCall => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            cx.sepc += 4;
            let result = sys_func::<SysFuncImpl>(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            match result {
                Ok(ret) => cx.x[10] = ret as usize,
                Err(id) => {
                    println!("[kernel] Unsupported syscall_id: {}", id);
                }
            }
        }
        Exception::StoreFault | Exception::StorePageFault => {
            println!("[kernel] PageFault in application, kernel killed it.");
            exit_and_run_next();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_and_run_next();
        }
        Exception::Interrupt(Interrupt::Timer) => {
            为下一次时钟中断定时();
            suspend_and_run_next();
        }
        _ => {}
    }
    let user_satp = 当前任务的地址空间.token();
    restore_context(CONTEXT_START_ADDR, user_satp);
}
