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
        let cx = task.borrow().get_trap_cx();
        let user_stack_top = task.borrow().user_stack_top();
        *cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
        let mut process_mut = process.borrow_mut();
        process_mut.tasks.insert(0, Rc::clone(&task));
        drop(process_mut);
        add_task(task);
        process
    }

    pub fn exec(&mut self, elf_data: &[u8]) {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        self.memory_set = memory_set;

        let task = self.main_task();
        let user_stack_top = task.borrow().user_stack_top();


        let cx = task.borrow().get_trap_cx();
        *cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
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

        let old_cx = self.main_task().borrow().get_trap_cx();
        let new_cx = process.borrow().main_task().borrow().get_trap_cx();
        *new_cx = old_cx.clone();

        add_task(task);
        process
    }
}

pub struct Task {
    pub process: Weak<RefCell<Process>>,
    pub is_exited: bool,
    pub tid: usize,
    pub pid: usize,
}

impl Task {
    pub fn new(process: Rc<RefCell<Process>>) -> Self {
        let tid = process.borrow_mut().alloc_tid();
        Self { 
            process: Rc::downgrade(&process),
            is_exited: false,
            tid,
            pid: process.borrow().pid.0,
        }
    }

    pub fn trap_cx_user_va(&self) -> usize {
        // TODO:
        // let process = self.process.upgrade().unwrap();
        // let pid = process.borrow().pid.0;
        0xFFFFFFFFFFFFE000 - self.pid * 0x10000 - self.tid * 0x1000
    }

    pub fn get_trap_cx(&self) -> &'static mut Context {
        let cx_va = self.trap_cx_user_va();
        unsafe {
            &mut *(cx_va as *mut Context)
        }
    }

    pub fn user_stack_top(&self) -> usize {
        0xFFFFFFFFFFFCF000 + (self.tid + 1) * 0x2000
    }
}
