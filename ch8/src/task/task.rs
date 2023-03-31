use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use exception::context::Context;
use super::id::{Pid, pid_alloc};
use crate::fs::{File, Stdin, Stdout};

pub struct Task {
    pub is_exited: bool,
    pub memory_set: 地址空间,
    pub pid: Pid,
    pub children: Vec<Rc<RefCell<Task>>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Rc<dyn File>>>,
}

impl Task {
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl Task {
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        let cx = memory_set.get_context();
        *cx = Context::app_init(
            entry_address,
            0xFFFFFFFFFFFFE000,
        );
        Self {
            is_exited: false,
            memory_set,
            pid: pid_alloc(),
            children: Vec::new(),
            exit_code: 0,
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

    pub fn exec(&mut self, elf_data: &[u8]) {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        let cx = memory_set.get_context();
        *cx = Context::app_init(
            entry_address,
            0xFFFFFFFFFFFFE000,
        );
        self.memory_set = memory_set;
    }

    pub fn fork(&mut self) -> Rc<RefCell<Self>> {
        let memory_set = 地址空间::复制地址空间(&self.memory_set);
        let mut new_fd_table: Vec<Option<Rc<dyn File>>> = Vec::new();
        for fd in self.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task = Rc::new(RefCell::new(
            Self {
                is_exited: false,
                memory_set,
                pid: pid_alloc(),
                children: Vec::new(),
                exit_code: 0,
                fd_table: new_fd_table,
            }
        ));
        self.children.push(Rc::clone(&task));
        task
    }
}
