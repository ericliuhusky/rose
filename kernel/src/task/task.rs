use crate::mm::memory_set::{UserSpace, USER_STACK_START_ADDR, USER_STACK_SIZE};
use crate::mutex::Mutex;
use crate::semaphore::Semaphore;
use alloc_ext::{rc::{MutRc, MutWeak}, collections::IDAllocDict};
use crate::exception::context::Context;
use super::{add_task, PROCESSES};
use crate::fs::{FileInterface, Stdin, Stdout};

pub struct Process {
    pub pid: Option<usize>,
    pub is_exited: bool,
    pub memory_set: UserSpace,
    pub fd_table: IDAllocDict<MutRc<dyn FileInterface>>,
    pub tasks: IDAllocDict<MutRc<Task>>,
    pub mutexs: IDAllocDict<MutRc<Mutex>>,
    pub semaphores: IDAllocDict<MutRc<Semaphore>>,
}

impl Process {
    pub fn main_task(&self) -> MutRc<Task> {
        self.tasks.get(0).unwrap().clone()
    }
}

impl Process {
    pub fn new(elf_data: &[u8]) -> MutRc<Self> {
        let (memory_set, entry_address) = UserSpace::new(elf_data);

        let mut fd_table: IDAllocDict<MutRc<dyn FileInterface>> = IDAllocDict::new();
        fd_table.insert(MutRc::new(Stdin));
        fd_table.insert(MutRc::new(Stdout));
        fd_table.insert(MutRc::new(Stdout));

        let mut process = MutRc::new(Self{
            pid: None,
            is_exited: false,
            memory_set,
            fd_table,
            tasks: IDAllocDict::new(),
            mutexs: IDAllocDict::new(),
            semaphores: IDAllocDict::new(),
        });
        let mut task = MutRc::new(Task::new(process.clone()));
        let tid = process.tasks.insert(task.clone());
        task.tid = Some(tid);
        let user_stack_top = task.user_stack_top();
        task.cx = Context::app_init(
            entry_address,
            user_stack_top,
        );
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
        let mut process = MutRc::new(Self {
            pid: None,
            is_exited: false,
            memory_set,
            fd_table: self.fd_table.clone(),
            tasks: IDAllocDict::new(),
            mutexs: IDAllocDict::new(),
            semaphores: IDAllocDict::new(),
        });
        let pid = unsafe {
            PROCESSES.insert(process.clone())
        };
        process.pid = Some(pid);
        let mut task = self.main_task().as_ref().clone();
        task.process = process.downgrade();
        let task = MutRc::new(task);
        process.tasks.insert(task.clone());

        add_task(task);
        process
    }
}

pub struct Task {
    pub process: MutWeak<Process>,
    pub is_exited: bool,
    pub tid: Option<usize>,
    pub cx: Context,
}

impl Task {
    pub fn new(process: MutRc<Process>) -> Self {
        Self { 
            process: process.downgrade(),
            is_exited: false,
            tid: None,
            cx: Context { x: [0; 32], sepc: 0 },
        }
    }

    pub fn user_stack_top(&self) -> usize {
        USER_STACK_START_ADDR + (self.tid.unwrap() + 1) * USER_STACK_SIZE
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
