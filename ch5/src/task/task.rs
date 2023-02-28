use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::vec::Vec;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use crate::trap::陷入上下文;
use super::pid::{进程标识符, 进程标识符管理器};

pub struct 任务 {
    pub 状态: 任务状态,
    pub 地址空间: 地址空间,
    pub 进程标识符: 进程标识符,
    pub 子进程列表: Vec<Rc<RefCell<任务>>>,
    pub 退出代码: i32
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
            地址空间,
            进程标识符: 进程标识符管理器::分配进程标识符(),
            子进程列表: Vec::new(),
            退出代码: 0
        }
    }

    pub fn exec(&mut self, elf文件数据: &[u8]) {
        let (地址空间, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf文件数据);
        let 上下文 = 地址空间.陷入上下文();
        *上下文 = 陷入上下文::应用初始上下文(
            应用入口地址,
            用户栈栈顶,
            unsafe { 内核地址空间.token() },
        );
        self.地址空间 = 地址空间;
    }

    pub fn fork(&mut self) -> Rc<RefCell<Self>> {
        let 地址空间 = 地址空间::复制地址空间(&self.地址空间);
        let 任务 = Rc::new(RefCell::new(
            Self {
                状态: 任务状态::就绪,
                地址空间,
                进程标识符: 进程标识符管理器::分配进程标识符(),
                子进程列表: Vec::new(),
                退出代码: 0
            }
        ));
        self.子进程列表.push(Rc::clone(&任务));
        任务
    }
}

#[derive(PartialEq)]
pub enum 任务状态 {
    就绪,
    运行,
    终止,
}
