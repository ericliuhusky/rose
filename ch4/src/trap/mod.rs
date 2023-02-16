use crate::syscall::syscall;
use core::arch::{asm, global_asm};
mod context;
mod scause;
pub use context::TrapContext;
use scause::{Trap, Exception, Interrupt};
use crate::timer::set_next_trigger;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::{
    TASK_MANAGER, exit_current_and_run_next, suspend_current_and_run_next,
};

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
    let cx = unsafe {
        TASK_MANAGER.current_task().get_trap_cx()
    };
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
    trap_return();
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = unsafe {
        TASK_MANAGER.current_task().page_table.token()
    };
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
