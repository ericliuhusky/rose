use super::context::任务上下文;
use core::arch::global_asm;

global_asm!(include_str!("switch.s"));

extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut 任务上下文, next_task_cx_ptr: *const 任务上下文);
}
