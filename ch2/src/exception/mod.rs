mod context;
use crate::{batch::应用管理器, segment::CONTEXT_START_ADDR};
use crate::syscall::sys_func;
use core::arch::global_asm;
pub use context::Context;
use riscv_register::{scause::{self, Exception}, stvec};

global_asm!(include_str!("save.s"));
global_asm!(include_str!("restore.s"));

extern "C" {
    fn __save();
    fn __restore();
}

pub fn restore_context() {
    use core::arch::asm;
    unsafe {
        let cx = &*(CONTEXT_START_ADDR as *const Context);
        asm!("csrw sepc, {}", in(reg) cx.sepc);
        __restore();
    }
}

#[no_mangle]
fn save_context() {
    use core::arch::asm;
    unsafe {
        let cx = &mut *(CONTEXT_START_ADDR as *mut Context);
        let mut t: usize;
        asm!("csrr {}, sepc", out(reg) t);
        cx.sepc = t;
        let mut t2: usize;
        asm!("csrr {}, sscratch", out(reg) t2);
        cx.x[2] = t2;
        exception_handler();
    }
}

pub fn 初始化() {
    // 设置异常处理入口地址为__save
    stvec::write(__save as usize);
}


/// 处理中断、异常或系统调用
pub fn exception_handler() {
    let 上下文 = unsafe { &mut *(CONTEXT_START_ADDR as *mut Context) };
    match scause::read() {
        Exception::UserEnvCall => {
            // ecall指令长度为4个字节，sepc加4以在sret的时候返回ecall指令的下一个指令继续执行
            上下文.sepc += 4;
            上下文.x[10] = sys_func(
                上下文.x[17],
                [
                    上下文.x[10],
                    上下文.x[11], 
                    上下文.x[12]
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
    restore_context();
}
