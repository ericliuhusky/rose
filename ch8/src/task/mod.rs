mod id;
pub mod task;

use self::task::{Task, Process};
use crate::mm::memory_set::CONTEXT_START_ADDR;
use alloc::{rc::Rc, vec::Vec};
use core::cell::{Ref, RefCell, RefMut};
use exception::{restore::restore_context, context::Context};
use sbi_call::shutdown;

pub struct TaskManager {
    pub current: Option<Rc<RefCell<Task>>>,
    ready_queue: Vec<Rc<RefCell<Task>>>,
}

impl TaskManager {
    fn current_task(&self) -> Rc<RefCell<Task>> {
        Rc::clone(self.current.as_ref().unwrap())
    }

    fn current_process(&self) -> Rc<RefCell<Process>> {
        let task = current_task();
        let task = task.borrow();
        task.process.upgrade().unwrap()
    }

    fn suspend_and_run_next(&mut self) {
        let previous = self.current.take().unwrap();
        self.ready_queue.push(previous);
        self.run_next();
    }

    fn exit_and_run_next(&mut self, exit_code: i32) {
        let previous = self.current.take().unwrap();
        let mut previous_mut = previous.borrow_mut();
        previous_mut.is_exited = true;
        let process = previous_mut.process.upgrade().unwrap();
        drop(previous_mut);

        if previous.borrow().tid == 0 {
            if process.borrow().pid.0 == 0 {
                println!("[Kernel] exit!");
                shutdown();
            }
    
            let mut process = process.borrow_mut();
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
        restore_context(current_trap_cx_user_va(), user_satp);
    }
}

pub static mut TASK_MANAGER: TaskManager = TaskManager {
    current: None,
    ready_queue: Vec::new(),
};

pub fn current_task() -> Rc<RefCell<Task>> {
    unsafe { TASK_MANAGER.current_task() }
}

pub fn current_process() -> Rc<RefCell<Process>> {
    unsafe { TASK_MANAGER.current_process() }
}

pub fn current_user_token() -> usize {
    let process = current_process();
    let process = process.borrow();
    process.memory_set.token()
}

pub fn current_trap_cx() -> &'static mut Context {
    let tid = current_task().borrow().tid;
    current_process().borrow().get_trap_cx(tid)
}

pub fn current_trap_cx_user_va() -> usize {
    current_task().borrow().trap_cx_user_va()
}

pub fn add_task(task: Rc<RefCell<Task>>) {
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
static mut ROOT_PROC: Option<Rc<RefCell<Process>>> = None;

pub fn add_initproc() {
    use crate::fs::open_file;
    let inode = open_file("initproc", false).unwrap();
    let elf_data = inode.read_all();
    let initproc = Process::new(&elf_data);
    unsafe {
        ROOT_PROC = Some(initproc);
    }
}
