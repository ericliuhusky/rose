use crate::syscall::系统调用;
use core::arch::{asm, global_asm};
mod context;
mod scause;
pub use context::陷入上下文;
use scause::{Trap, Exception, Interrupt};
use crate::timer::为下一次时钟中断定时;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::任务管理器;
use crate::格式化输出并换行;

global_asm!(include_str!("trap2.s"));

/// 设置trap入口地址为__trap_entry
pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    // stvec寄存器设置中断跳转地址
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) trap_from_kernel as usize);
    }
}

fn set_user_trap_entry() {
    // stvec寄存器设置中断跳转地址
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) TRAMPOLINE as usize);
    }
}


#[no_mangle] 
/// 处理中断、异常或系统调用
pub fn trap_handler() {
    set_kernel_trap_entry();
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
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = 任务管理器::当前页表().token();
    extern "C" {
        fn __trap_entry();
        fn __restore();
    }
    let restore_va = __restore as usize - __trap_entry as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}
