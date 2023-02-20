mod task;

use crate::loader::{读取应用数目, 读取应用数据};
use crate::mm::PageTable;
use alloc::vec::Vec;
use task::{任务, 任务状态};
use crate::格式化输出并换行;
use crate::终止::终止;

pub struct 任务管理器 {
    任务数目: usize,
    任务列表: Vec<任务>,
    当前任务索引: isize,
}

impl 任务管理器 {
    pub fn 初始化() {
        let 任务数目 = 读取应用数目();
        let mut 任务列表 = Vec::new();
        for 应用索引 in 0..任务数目 {
            任务列表.push(任务::new(读取应用数据(应用索引)))
        }
        unsafe {
            任务管理器 = 任务管理器 {
                任务数目,
                任务列表,
                当前任务索引: -1
            };
        }
    }

    pub fn 当前任务(&mut self) -> &mut 任务 {
        &mut self.任务列表[self.当前任务索引 as usize]
    }

    pub fn 暂停并运行下一个任务() {
        unsafe {
            任务管理器.当前任务().状态 = 任务状态::就绪;
            Self::运行下一个任务();
        }
    }

    pub fn 终止并运行下一个任务() {
        unsafe {
            任务管理器.当前任务().状态 = 任务状态::终止;
            Self::运行下一个任务();
        }
    }

    fn 查找下一个就绪任务(&mut self) -> Option<&mut 任务> {
        let 下一个任务索引 = (self.当前任务索引 + 1) as usize;
        let 下一个就绪任务索引 = (下一个任务索引..下一个任务索引 + self.任务数目)
            .map(|任务索引| 任务索引 % self.任务数目)
            .find(|任务索引| self.任务列表[*任务索引].状态 == 任务状态::就绪);
        if let Some(下一个就绪任务索引) = 下一个就绪任务索引 {
            self.当前任务索引 = 下一个就绪任务索引 as isize;
            Some(&mut self.任务列表[下一个就绪任务索引])
        } else {
            None
        }
    }

    pub fn 运行下一个任务() {
        unsafe {
            if let Some(下一个任务) = 任务管理器.查找下一个就绪任务() {
                下一个任务.状态 = 任务状态::运行;
                extern "C" {
                    fn __restore(user_satp: usize);
                }
                __restore(下一个任务.页表.token());
            } else {
                格式化输出并换行!("[Kernel] All applications completed!");
                终止();
            }
        }
    }

    pub fn 当前页表() -> &'static PageTable {
        unsafe {
            &任务管理器.当前任务().页表
        }
    }
}

static mut 任务管理器: 任务管理器 = 任务管理器 {
    任务数目: 0,
    任务列表: Vec::new(),
    当前任务索引: 0
};
