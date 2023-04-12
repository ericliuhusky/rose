use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use crate::mm::memory_set::{UserSpace, USER_STACK_START_ADDR, USER_STACK_SIZE};
use crate::mutex::Mutex;
use crate::semaphore::Semaphore;
use mutrc::{MutRc, MutWeak};
use exception::context::Context;
use super::add_task;
use super::id::{Pid, pid_alloc, IDAllocator, IDAllocDict};
use crate::fs::{File, Stdin, Stdout};

pub struct Process {
    pub pid: Pid,
    pub is_exited: bool,
    pub memory_set: UserSpace,
    pub children: Vec<MutRc<Process>>,
    pub fd_table: Vec<Option<MutRc<dyn File>>>,
    pub tasks: BTreeMap<usize, MutRc<Task>>,
    pub tid_allocator: IDAllocator,
    pub mutexs: IDAllocDict<MutRc<Mutex>>,
    pub semaphores: IDAllocDict<MutRc<Semaphore>>,
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
        let (memory_set, entry_address) = UserSpace::new(elf_data);
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
            mutexs: IDAllocDict::new(),
            semaphores: IDAllocDict::new(),
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
        let (memory_set, entry_address) = UserSpace::new(elf_data);
        self.memory_set = memory_set;

        let mut task = self.main_task();
        let user_stack_top = task.user_stack_top();

        task.cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
    }

    pub fn fork(&mut self) -> MutRc<Self> {
        let memory_set = self.memory_set.clone();
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
            mutexs: IDAllocDict::new(),
            semaphores: IDAllocDict::new(),
        });
        self.children.push(process.clone());
        let mut task = self.main_task().as_ref().clone();
        // MARK: 确保下一个分配的tid是1，是0会覆盖主线程
        process.alloc_tid();
        task.process = process.downgrade();
        let task = MutRc::new(task);
        process.tasks.insert(0, task.clone());

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

    pub fn user_stack_top(&self) -> usize {
        USER_STACK_START_ADDR + (self.tid + 1) * USER_STACK_SIZE
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        let process = self.process.upgrade().unwrap().downgrade();
        Self {
            process,
            is_exited: self.is_exited,
            tid: self.tid,
            cx: self.cx.clone(),
        }
    }
}
