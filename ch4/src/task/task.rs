use crate::mm::memory_set::地址空间;
use crate::mm::page_table::PageTable;
use crate::trap::{陷入上下文};

pub struct 任务 {
    pub 状态: 任务状态,
    pub 页表: PageTable,
}

impl 任务 {
    pub fn new(elf_data: &[u8]) -> Self {
        let (页表, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf_data);
        let 状态 = 任务状态::就绪;
        let trap_cx = 页表.translated_trap_context();
        *trap_cx = 陷入上下文::应用初始上下文(
            应用入口地址,
            用户栈栈顶,
            地址空间::内核地址空间token(),
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
