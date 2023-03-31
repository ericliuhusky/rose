use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use exception::context::Context;
use super::id::{Pid, pid_alloc};
use crate::fs::{File, Stdin, Stdout};

pub struct 任务 {
    pub is_exited: bool,
    pub 地址空间: 地址空间,
    pub 进程标识符: Pid,
    pub 子进程列表: Vec<Rc<RefCell<任务>>>,
    pub 终止代码: i32,
    pub fd_table: Vec<Option<Rc<dyn File>>>,
}

impl 任务 {
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl 任务 {
    pub fn 新建(elf文件数据: &[u8]) -> Self {
        let (地址空间, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf文件数据);
        let 上下文 = 地址空间.陷入上下文();
        *上下文 = Context::app_init(
            应用入口地址,
            用户栈栈顶,
        );
        Self {
            is_exited: false,
            地址空间,
            进程标识符: pid_alloc(),
            子进程列表: Vec::new(),
            终止代码: 0,
            fd_table: vec![
                // 0 -> stdin
                Some(Rc::new(Stdin)),
                // 1 -> stdout
                Some(Rc::new(Stdout)),
                // 2 -> stderr
                Some(Rc::new(Stdout)),
            ],
        }
    }

    pub fn exec(&mut self, elf文件数据: &[u8]) {
        let (地址空间, 用户栈栈顶, 应用入口地址) = 地址空间::新建应用地址空间(elf文件数据);
        let 上下文 = 地址空间.陷入上下文();
        *上下文 = Context::app_init(
            应用入口地址,
            用户栈栈顶,
        );
        self.地址空间 = 地址空间;
    }

    pub fn fork(&mut self) -> Rc<RefCell<Self>> {
        let 地址空间 = 地址空间::复制地址空间(&self.地址空间);
        let mut new_fd_table: Vec<Option<Rc<dyn File>>> = Vec::new();
        for fd in self.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let 任务 = Rc::new(RefCell::new(
            Self {
                is_exited: false,
                地址空间,
                进程标识符: pid_alloc(),
                子进程列表: Vec::new(),
                终止代码: 0,
                fd_table: new_fd_table,
            }
        ));
        self.子进程列表.push(Rc::clone(&任务));
        任务
    }
}
