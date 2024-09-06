use crate::exception::restore::restore_context;
use crate::syscall::syscall;
use crate::task::{current_process, current_task, exit_and_run_next, suspend_and_run_next};
use crate::timer::为下一次时钟中断定时;
use riscv::register::scause::Trap;
use riscv::register::scause::{self, Exception, Interrupt};

#[no_mangle]
/// 处理中断、异常或系统调用
pub fn exception_handler() {
    // let 当前任务的地址空间 = 任务管理器::当前任务().borrow().地址空间;
    let mut task = current_task();
    let 上下文 = &mut task.cx;
    match scause::read().cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.sepc += 4;
            let result = syscall(上下文.x[17], [上下文.x[10], 上下文.x[11], 上下文.x[12]]);
            match result {
                Ok(ret) => 上下文.x[10] = ret,
                Err(id) => {
                    println!("[kernel] Unsupported syscall_id: {}", id);
                }
            }
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            exit_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            为下一次时钟中断定时();
            suspend_and_run_next();
        }
        _ => {}
    }
    let token = current_process().memory_set.page_table.satp();
    let task = current_task();
    restore_context(&task.cx, token);
}
