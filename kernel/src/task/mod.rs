mod id;
pub mod task;

use mutrc::MutRc;
use self::task::{Task, Process};
use alloc::{rc::Rc, vec::Vec};
use exception::{restore::restore_context, context::Context};
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

    fn exit_and_run_next(&mut self, exit_code: i32) {
        let mut previous = self.current.take().unwrap();
        previous.is_exited = true;
        let mut process = previous.process.upgrade().unwrap();

        if previous.tid == 0 {
            if process.pid.0 == 0 {
                println!("[Kernel] exit!");
                shutdown();
            }
    
            process.is_exited = true;
            process.children.clear();
            process.tasks.clear();
        }

        drop(process);
        self.run_next();
    }

    fn run_next(&mut self) {
        let next = self.ready_queue.remove(0);        
        self.current = Some(next);
        let user_satp = current_user_token();
        restore_context(current_trap_cx(), user_satp);
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
    process.memory_set.token()
}

pub fn current_trap_cx() -> &'static mut Context {
    current_task().get_trap_cx()
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

pub fn exit_and_run_next(exit_code: i32) {
    unsafe {
        TASK_MANAGER.exit_and_run_next(exit_code);
    }
}

// TODO: 必须持有根进程才不会被释放
static mut ROOT_PROC: Option<MutRc<Process>> = None;

pub fn add_initproc() {
    use crate::fs::open_file;
    let inode = open_file("initproc", false).unwrap();
    let elf_data = inode.read_all();
    let initproc = Process::new(&elf_data);
    unsafe {
        ROOT_PROC = Some(initproc);
    }
}
