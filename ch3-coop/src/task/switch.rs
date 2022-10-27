use super::context::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.s"));

extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
