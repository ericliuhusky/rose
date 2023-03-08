use crate::loader::将应用初始上下文压入内核栈后的栈顶;
use alloc::vec::Vec;
use sbi_call::shutdown;

struct 任务 {
    状态: 任务状态,
    内核栈栈顶: usize
}

#[derive(PartialEq)]
enum 任务状态 {
    就绪,
    运行,
    终止,
}

pub struct 任务管理器 {
    任务数目: usize,
    任务列表: Vec<任务>,
    当前任务索引: isize,
}

impl 任务管理器 {
    pub fn 初始化() {
        let 任务数目 = loader::read_app_num();
        let mut 任务列表 = Vec::new();
        for i in 0..任务数目 {
            任务列表.push(任务 {
                内核栈栈顶: 将应用初始上下文压入内核栈后的栈顶(i),
                状态: 任务状态::就绪
            })
        }
        unsafe {
            任务管理器 = 任务管理器 {
                任务数目,
                任务列表,
                当前任务索引: -1
            };
        }
    }

    fn 当前任务(&mut self) -> &mut 任务 {
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
                    fn __restore(cx_addr: usize);
                }
                __restore(下一个任务.内核栈栈顶);
            } else {
                println!("[Kernel] All applications completed!");
                shutdown();
            }
        }
    }
}

static mut 任务管理器: 任务管理器 = 任务管理器 {
    任务数目: 0,
    任务列表: Vec::new(),
    当前任务索引: 0
};
