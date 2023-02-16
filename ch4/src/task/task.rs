//! Types related to task management

use super::TaskContext;
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mm::{MemorySet, PhysPageNum, VirtPageNum, KERNEL_SPACE, PageTable};
use crate::trap::{trap_handler, TrapContext};

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub page_table: PageTable,
    pub trap_cx_ppn: PhysPageNum,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (page_table, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = page_table
            .translate(VirtPageNum::from(TRAP_CONTEXT));
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        unsafe {
            KERNEL_SPACE.insert_framed_area(
                kernel_stack_bottom..kernel_stack_top,
                false,
            );
        }
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            page_table,
            trap_cx_ppn,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            unsafe { KERNEL_SPACE.page_table.token() },
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}
