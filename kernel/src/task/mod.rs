pub mod task;

use alloc_ext::{rc::MutRc, collections::IDAllocDict};
use self::task::{Task, Process};
use alloc::vec::Vec;
use crate::exception::restore::restore_context;
use sbi_call::shutdown;

pub struct TaskManager {
    pub current: Option<MutRc<Task>>,
    ready_queue: Vec<MutRc<Task>>,
}

impl TaskManager {
    fn current_task(&self) -> MutRc<Task> {
        self.current.as_ref().unwrap().clone()
    }

    fn current_process(&self) -> MutRc<Process> {
        let task = current_task();
        task.process.upgrade().unwrap()
    }

    fn suspend_and_run_next(&mut self) {
        let previous = self.current.take().unwrap();
        self.ready_queue.push(previous);
        self.run_next();
    }

    fn exit_and_run_next(&mut self) {
        let mut previous = self.current.take().unwrap();
        previous.is_exited = true;
        let mut process = previous.process.upgrade().unwrap();

        if previous.tid.unwrap() == 0 {
            if process.pid.unwrap() == 0 {
                println!("[Kernel] exit!");
                shutdown();
            }
    
            process.is_exited = true;
        }

        drop(process);
        self.run_next();
    }

    fn block_and_run_next(&mut self) {
        self.current.take().unwrap();
        self.run_next();
    }

    fn run_next(&mut self) {
        let next = self.ready_queue.remove(0);        
        self.current = Some(next);
        let user_satp = current_user_token();
        let task = current_task();
        restore_context(&task.cx, user_satp);
    }
}

pub static mut TASK_MANAGER: TaskManager = TaskManager {
    current: None,
    ready_queue: Vec::new(),
};

pub fn current_task() -> MutRc<Task> {
    unsafe { TASK_MANAGER.current_task() }
}

pub fn current_process() -> MutRc<Process> {
    unsafe { TASK_MANAGER.current_process() }
}

pub fn current_user_token() -> usize {
    let process = current_process();
    process.memory_set.page_table.satp()
}

pub fn add_task(task: MutRc<Task>) {
    unsafe {
        TASK_MANAGER.ready_queue.push(task);
    }
}

pub fn run_next() {
    unsafe {
        TASK_MANAGER.run_next();
    }
}

pub fn suspend_and_run_next() {
    unsafe {
        TASK_MANAGER.suspend_and_run_next();
    }
}

pub fn exit_and_run_next() {
    unsafe {
        TASK_MANAGER.exit_and_run_next();
    }
}

pub fn block_and_run_next() {
    unsafe {
        TASK_MANAGER.block_and_run_next();
    }
}

pub fn wakeup_task(task: MutRc<Task>) {
    add_task(task);
}

pub static mut PROCESSES: IDAllocDict<MutRc<Process>> = IDAllocDict::new();

pub fn add_initproc() {
    let f = fs::open("initproc", false).unwrap();
    let elf_data = f._read();
    let mut initproc = Process::new(&elf_data);
    let pid = unsafe {
        PROCESSES.insert(initproc.clone())
    };
    initproc.pid = Some(pid);
}
