use alloc::vec::Vec;
use sbi_call::shutdown;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use crate::trap::Context;

pub struct 任务 {
    状态: 任务状态,
    pub 地址空间: 地址空间,
}

impl 任务 {
    fn 新建(elf文件数据: &[u8]) -> Self {
        let (地址空间, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf文件数据);
        let 上下文 = 地址空间.陷入上下文();
        *上下文 = Context::app_init(
            应用入口地址,
            用户栈栈顶,
        );
        Self {
            状态: 任务状态::就绪,
            地址空间
        }
    }
}

#[derive(PartialEq)]
enum 任务状态 {
    就绪,
    运行,
    终止,
}


pub struct 任务管理器 {
    任务列表: Vec<任务>,
    当前任务索引: isize,
}

impl 任务管理器 {
    pub fn 初始化() {
        let 任务数目 = loader::read_app_num();
        let 任务列表 = (0..任务数目)
            .map(|应用索引| {
                任务::新建(loader::read_app_data(应用索引))
            })
            .collect();
        unsafe {
            任务管理器 = 任务管理器 {
                任务列表,
                当前任务索引: -1
            };
        }
    }

    pub fn 当前任务() -> &'static mut 任务 {
        unsafe {
            &mut 任务管理器.任务列表[任务管理器.当前任务索引 as usize]
        }
    }

    pub fn 暂停并运行下一个任务() {
        Self::当前任务().状态 = 任务状态::就绪;
        Self::运行下一个任务();
    }

    pub fn 终止并运行下一个任务() {
        Self::当前任务().状态 = 任务状态::终止;
        Self::运行下一个任务();
    }

    fn 查找下一个就绪任务(&mut self) -> Option<&mut 任务> {
        let 下一个任务索引 = (self.当前任务索引 + 1) as usize;
        let 任务数目 = self.任务列表.len();
        let 下一个就绪任务索引 = (下一个任务索引..下一个任务索引 + 任务数目)
            .map(|任务索引| {
                任务索引 % 任务数目
            })
            .find(|任务索引| {
                self.任务列表[*任务索引].状态 == 任务状态::就绪
            });
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
                __restore(下一个任务.地址空间.token());
            } else {
                println!("[Kernel] All applications completed!");
                shutdown();
            }
        }
    }
}

static mut 任务管理器: 任务管理器 = 任务管理器 {
    任务列表: Vec::new(),
    当前任务索引: 0
};
