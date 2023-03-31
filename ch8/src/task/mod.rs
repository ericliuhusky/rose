mod id;
pub mod task;

use self::task::Task;
use crate::mm::memory_set::CONTEXT_START_ADDR;
use alloc::{rc::Rc, vec::Vec};
use core::cell::{Ref, RefCell, RefMut};
use exception::restore::restore_context;
use sbi_call::shutdown;

pub struct TaskManager {
    pub current: Option<Rc<RefCell<Task>>>,
    ready_queue: Vec<Rc<RefCell<Task>>>,
}

impl TaskManager {
    fn current(&self) -> Rc<RefCell<Task>> {
        Rc::clone(self.current.as_ref().unwrap())
    }

    fn suspend_and_run_next(&mut self) {
        let previous = self.current.take().unwrap();
        self.ready_queue.push(previous);
        self.run_next();
    }

    fn exit_and_run_next(&mut self, exit_code: i32) {
        if self.current().borrow().pid.0 == 0 {
            println!("[Kernel] exit!");
            shutdown();
        }

        let task = self.current();
        let mut task = task.borrow_mut();
        task.is_exited = true;
        task.exit_code = exit_code;
        task.children.clear();
        drop(task);

        self.run_next();
    }

    fn run_next(&mut self) {
        let next = self.ready_queue.remove(0);
        let user_satp = next.borrow().memory_set.token();
        self.current = Some(next);
        restore_context(CONTEXT_START_ADDR, user_satp);
    }
}

pub static mut TASK_MANAGER: TaskManager = TaskManager {
    current: None,
    ready_queue: Vec::new(),
};

pub fn current_task() -> Rc<RefCell<Task>> {
    unsafe { TASK_MANAGER.current() }
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

pub fn add_initproc() {
    use crate::fs::open_file;
    let inode = open_file("initproc", false).unwrap();
    let elf_data = inode.read_all();
    add_task(Rc::new(RefCell::new(Task::new(&elf_data))))
}
