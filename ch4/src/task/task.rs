use crate::mm::{MemorySet, KERNEL_SPACE, PageTable};
use crate::trap::{陷入上下文};

pub struct 任务 {
    pub 状态: 任务状态,
    pub 页表: PageTable,
}

impl 任务 {
    pub fn new(elf_data: &[u8]) -> Self {
        let (页表, 用户栈栈顶, 应用入口地址) = MemorySet::from_elf(elf_data);
        let 状态 = 任务状态::就绪;
        let trap_cx = 页表.translated_trap_context();
        *trap_cx = 陷入上下文::应用初始上下文(
            应用入口地址,
            用户栈栈顶,
            unsafe { KERNEL_SPACE.page_table.token() },
        );
        Self {
            状态,
            页表,
        }
    }
}

#[derive(PartialEq)]
pub enum 任务状态 {
    就绪,
    运行,
    终止,
}
