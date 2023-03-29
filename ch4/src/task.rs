use alloc::collections::VecDeque;
use sbi_call::shutdown;
use crate::mm::USER_SATP;
use crate::mm::memory_set::{地址空间, 内核地址空间};
use exception::context::Context;
use exception::restore::restore_context;

pub struct Task {
    pub memory_set: 地址空间,
}

impl Task {
    fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_stack_top, enrty_address) = 地址空间::新建应用地址空间(elf_data);
        let cx = memory_set.陷入上下文();
        *cx = Context::app_init(
            enrty_address,
            user_stack_top,
        );
        Self {
            memory_set
        }
    }
}

pub struct TaskManager {
    ready_queue: VecDeque<Task>,
    current: Option<Task>,
}

impl TaskManager {
    fn new() -> Self {
        let n = loader::read_app_num();
        let tasks = (0..n)
            .map(|i| {
                Task::new(loader::read_app_data(i))
            })
            .collect();
        Self {
            ready_queue: tasks,
            current: None,
        }
    }

    fn suspend_and_run_next(&mut self) {
        let previous = self.current.take().unwrap();
        self.ready_queue.push_back(previous);
        self.run_next();
    }

    fn exit_and_run_next(&mut self) {
        self.current.take().unwrap();
        self.run_next();
    }

    fn run_next(&mut self) {
        if let Some(next) = self.ready_queue.pop_front() {
            unsafe {
                USER_SATP = next.memory_set.token();
            }
            self.current = Some(next);
            restore_context();
        } else {
            println!("[Kernel] All applications completed!");
            shutdown();
        }
    }
}

static mut TASK_MANAGER: Option<TaskManager> = None;

pub fn init() {
    unsafe {
        TASK_MANAGER = Some(TaskManager::new());
    }
}

pub fn current_task() -> &'static Task {
    unsafe {
        TASK_MANAGER.as_ref().unwrap().current.as_ref().unwrap()
    }
}

pub fn suspend_and_run_next() {
    unsafe {
        TASK_MANAGER.as_mut().unwrap().suspend_and_run_next();
    }
}

pub fn exit_and_run_next() {
    unsafe {
        TASK_MANAGER.as_mut().unwrap().exit_and_run_next();
    }
}

pub fn run_next() {
    unsafe {
        TASK_MANAGER.as_mut().unwrap().run_next();
    }
}
