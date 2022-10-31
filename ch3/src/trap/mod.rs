use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::syscall::syscall;
use core::arch::{global_asm};
mod context;
mod scause;
pub use context::TrapContext;
use scause::{Trap, Exception, Interrupt};
use crate::timer::set_next_trigger;

global_asm!(include_str!("trap.s"));

/// 设置trap入口地址为__trap_entry
pub fn init() {
    extern "C" {
        fn __trap_entry();
    }
    // stvec寄存器设置中断跳转地址
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) __trap_entry as usize);
    }
}


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn trap_handler(cx: &mut TrapContext) {
    match scause::read().cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // sepc寄存器记录触发中断的指令地址
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            
        }
    }
}
