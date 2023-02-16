//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
}

/// Global variable: TASK_MANAGER
pub static mut TASK_MANAGER: TaskManager = TaskManager {
    num_app: 0,
    tasks: [TaskControlBlock {task_status: TaskStatus::UnInit, task_cx: TaskContext {ra:0, sp:0, s:[0;12]}}; MAX_APP_NUM],
    current_task: 0
};

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&mut self) -> ! {
        let task0 = &mut self.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&mut self) {
        let current = self.current_task;
        self.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&mut self) {
        let current = self.current_task;
        self.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return app id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let current = self.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| self.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&mut self) {
        if let Some(next) = self.find_next_task() {
            let current = self.current_task;
            self.tasks[next].task_status = TaskStatus::Running;
            self.current_task = next;
            let current_task_cx_ptr = &mut self.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &self.tasks[next].task_cx as *const TaskContext;
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            println!("[Kernel] All applications completed!");

            crate::exit::exit();
        }
    }
}

/// run first task
pub fn run_first_task() {
    let num_app = get_num_app();
    let mut tasks = [TaskControlBlock {
        task_cx: TaskContext::zero_init(),
        task_status: TaskStatus::UnInit,
    }; MAX_APP_NUM];
    for (i, task) in tasks.iter_mut().enumerate() {
        task.task_cx = TaskContext::goto_restore(init_app_cx(i));
        task.task_status = TaskStatus::Ready;
    }
    unsafe {
        TASK_MANAGER = TaskManager {
            num_app,
            tasks,
            current_task: 0
        };
        TASK_MANAGER.run_first_task();
    }
}

/// rust next task
fn run_next_task() {
    unsafe {
        TASK_MANAGER.run_next_task();
    }
}

/// suspend current task
fn mark_current_suspended() {
    unsafe {
        TASK_MANAGER.mark_current_suspended();
    }
}

/// exit current task
fn mark_current_exited() {
    unsafe {
        TASK_MANAGER.mark_current_exited();
    }
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
