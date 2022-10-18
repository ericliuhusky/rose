use crate::batch::run_next_app;
use crate::syscall::syscall;
use crate::csr::{scause, stvec};
use core::arch::{global_asm};
mod context;
use context::TrapContext;

global_asm!(include_str!("__save_regs.s"));
global_asm!(include_str!("__restore_regs.s"));

/// 设置trap入口地址为__save_regs
pub fn init() {
    extern "C" {
        fn __save_regs();
    }
    // stvec寄存器设置中断跳转地址
    stvec::write(__save_regs as usize);
}


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn trap_handler(cx: &mut TrapContext) {
    cx.save_csr();

    match scause::read().cause() {
        scause::Exception::UserEnvCall => {
            // sepc寄存器记录触发中断的指令地址
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        scause::Exception::StoreFault | scause::Exception::StorePageFault => {
            println!("[kernel] PageFault in application, kernel killed it.");
            run_next_app();
        }
        scause::Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }
        _ => {
            
        }
    }

    cx.restore_csr();

    extern "C" {
        fn __restore_regs(cx: &mut TrapContext);
    }
    unsafe {
        __restore_regs(cx);
    }
}
