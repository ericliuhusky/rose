use alloc::rc::Weak;
use alloc::{rc::Rc, collections::BTreeMap};
use alloc::vec;
use alloc::vec::Vec;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use mutrc::{MutRc, MutWeak};
use exception::context::Context;
use super::add_task;
use super::id::{Pid, pid_alloc, IDAllocator};
use crate::fs::{File, Stdin, Stdout};

pub struct Process {
    pub pid: Pid,
    pub is_exited: bool,
    pub memory_set: 地址空间,
    pub children: Vec<MutRc<Process>>,
    pub fd_table: Vec<Option<MutRc<dyn File>>>,
    pub tasks: BTreeMap<usize, MutRc<Task>>,
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

    pub fn main_task(&self) -> MutRc<Task> {
        self.tasks.get(&0).unwrap().clone()
    }
}

impl Process {
    pub fn new(elf_data: &[u8]) -> MutRc<Self> {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        let mut process = MutRc::new(Self{
            pid: pid_alloc(),
            is_exited: false,
            memory_set,
            children: Vec::new(),
            fd_table: vec![
                Some(MutRc::new(Stdin)),
                Some(MutRc::new(Stdout)),
                Some(MutRc::new(Stdout)),
            ],
            tasks: BTreeMap::new(),
            tid_allocator: IDAllocator::new(),
        });
        let mut task = MutRc::new(Task::new(process.clone()));
        let user_stack_top = task.user_stack_top();
        task.cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
        process.tasks.insert(0, task.clone());
        add_task(task);
        process
    }

    pub fn exec(&mut self, elf_data: &[u8]) {
        let (memory_set, entry_address) = 地址空间::新建应用地址空间(elf_data);
        self.memory_set = memory_set;

        let mut task = self.main_task();
        let user_stack_top = task.user_stack_top();

        task.cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
    }

    pub fn fork(&mut self) -> MutRc<Self> {
        let memory_set = 地址空间::复制地址空间(&self.memory_set);
        let mut new_fd_table: Vec<Option<MutRc<dyn File>>> = Vec::new();
        for fd in self.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let mut process = MutRc::new(Self {
            pid: pid_alloc(),
            is_exited: false,
            memory_set,
            children: Vec::new(),
            fd_table: new_fd_table,
            tasks: BTreeMap::new(),
            tid_allocator: IDAllocator::new(),
        });
        self.children.push(process.clone());
        let task = MutRc::new(Task::new(process.clone()));
        process.tasks.insert(0, task.clone());

        let old_task = self.main_task();
        let mut new_task = process.main_task();
        new_task.cx = old_task.cx.clone();

        add_task(task);
        process
    }
}

pub struct Task {
    pub process: MutWeak<Process>,
    pub is_exited: bool,
    pub tid: usize,
    pub cx: Context,
}

impl Task {
    pub fn new(mut process: MutRc<Process>) -> Self {
        let tid = process.alloc_tid();
        Self { 
            process: process.downgrade(),
            is_exited: false,
            tid,
            cx: Context { x: [0; 32], sepc: 0 },
        }
    }

    pub fn get_trap_cx(&self) -> &'static mut Context {
        let cx_va = &self.cx as *const Context as usize;
        unsafe {
            &mut *(cx_va as *mut Context)
        }
    }

    pub fn user_stack_top(&self) -> usize {
        0xFFFFFFFFFFFCF000 + (self.tid + 1) * 0x2000
    }
}
