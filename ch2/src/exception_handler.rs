use crate::syscall::SysFuncImpl;
use crate::{batch::应用管理器, segment::CONTEXT_START_ADDR};
use exception::context::Context;
use riscv_register::{
    scause::{self, Exception},
    stvec,
};
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
            应用管理器::运行下一个应用();
        }
        Exception::IllegalInstruction => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            应用管理器::运行下一个应用();
        }
        _ => {}
    }
}
