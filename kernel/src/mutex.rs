use crate::task::{
    block_and_run_next, current_task, task::Task, wakeup_task,
};
use alloc::collections::VecDeque;
use alloc_ext::mutrc::MutRc;

pub struct Mutex {
    locked: bool,
    wait_queue: VecDeque<MutRc<Task>>,
}

impl Mutex {
    pub fn new() -> Self {
        Self {
            locked: false,
            wait_queue: VecDeque::new(),
        }
    }
}

impl Mutex {
    pub fn lock(&mut self) {
        if self.locked {
            self.wait_queue.push_back(current_task());
            block_and_run_next();
        } else {
            self.locked = true;
        }
    }

    pub fn unlock(&mut self) {
        if let Some(task) = self.wait_queue.pop_front() {
            wakeup_task(task);
        } else {
            self.locked = false;
        }
    }
}
