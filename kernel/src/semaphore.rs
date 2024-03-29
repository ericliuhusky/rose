use crate::task::{block_and_run_next, current_task, task::Task, wakeup_task};
use alloc::collections::VecDeque;
use alloc_ext::rc::MutRc;

pub struct Semaphore {
    count: isize,
    wait_queue: VecDeque<MutRc<Task>>,
}

impl Semaphore {
    pub fn new(res_count: usize) -> Self {
        Self {
            count: res_count as isize,
            wait_queue: VecDeque::new(),
        }
    }

    pub fn down(&mut self) {
        self.count -= 1;
        if self.count < 0 {
            self.wait_queue.push_back(current_task());
            block_and_run_next();
        }
    }

    pub fn up(&mut self) {
        self.count += 1;
        if self.count <= 0 {
            if let Some(task) = self.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }
    }
}
