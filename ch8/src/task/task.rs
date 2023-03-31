use core::cell::RefCell;
use alloc::rc::Weak;
use alloc::{rc::Rc, collections::BTreeMap};
use alloc::vec;
use alloc::vec::Vec;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use exception::context::Context;
use super::add_task;
use super::id::{Pid, pid_alloc, IDAllocator};
use crate::fs::{File, Stdin, Stdout};

pub struct Process {
    pub pid: Pid,
    pub is_exited: bool,
    pub memory_set: 地址空间,
    pub children: Vec<Rc<RefCell<Process>>>,
    pub fd_table: Vec<Option<Rc<dyn File>>>,
    pub tasks: BTreeMap<usize, Rc<RefCell<Task>>>,
    pub tid_allocator: IDAllocator,
}

impl Process {
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }

    pub fn alloc_tid(&mut self) -> usize {
        self.tid_allocator.alloc()
    }

    pub fn main_task(&self) -> Rc<RefCell<Task>> {
        self.tasks.get(&0).unwrap().clone()
    }
}

impl Process {
    pub fn new(elf_data: &[u8]) -> Rc<RefCell<Self>> {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        let process = Rc::new(RefCell::new(Self{
            pid: pid_alloc(),
            is_exited: false,
            memory_set,
            children: Vec::new(),
            fd_table: vec![
                Some(Rc::new(Stdin)),
                Some(Rc::new(Stdout)),
                Some(Rc::new(Stdout)),
            ],
            tasks: BTreeMap::new(),
            tid_allocator: IDAllocator::new(),
        }));
        let task = Rc::new(RefCell::new(Task::new(Rc::clone(&process))));
        let cx = process.borrow().memory_set.get_context();
        *cx = Context::app_init(
            entry_address,
            0xFFFFFFFFFFFFE000,
        );
        let mut process_mut = process.borrow_mut();
        process_mut.tasks.insert(0, Rc::clone(&task));
        drop(process_mut);
        add_task(task);
        process
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
        let process = Rc::new(RefCell::new(Self {
            pid: pid_alloc(),
            is_exited: false,
            memory_set,
            children: Vec::new(),
            fd_table: new_fd_table,
            tasks: BTreeMap::new(),
            tid_allocator: IDAllocator::new(),
        }));
        self.children.push(Rc::clone(&process));
        let task = Rc::new(RefCell::new(Task::new(Rc::clone(&process))));
        let mut process_mut = process.borrow_mut();
        process_mut.tasks.insert(0, Rc::clone(&task));
        drop(process_mut);
        add_task(task);
        process
    }
}

pub struct Task {
    pub process: Weak<RefCell<Process>>,
}

impl Task {
    pub fn new(process: Rc<RefCell<Process>>) -> Self {
        Self { 
            process: Rc::downgrade(&process),
        }
    }
}
