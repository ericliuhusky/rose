use crate::mm::memory_set::{地址空间, 内核地址空间};
use crate::trap::陷入上下文;

pub struct 任务 {
    pub 状态: 任务状态,
    pub 地址空间: 地址空间,
}

impl 任务 {
    pub fn 新建(elf文件数据: &[u8]) -> Self {
        let (地址空间, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf文件数据);
        let 上下文 = 地址空间.陷入上下文();
        *上下文 = 陷入上下文::应用初始上下文(
            应用入口地址,
            用户栈栈顶,
            unsafe { 内核地址空间.token() },
        );
        Self {
            状态: 任务状态::就绪,
            地址空间
        }
    }
}

#[derive(PartialEq)]
pub enum 任务状态 {
    就绪,
    运行,
    终止,
}
