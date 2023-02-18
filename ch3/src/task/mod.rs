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

use crate::loader::{读取应用程序数目, init_app_cx};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use crate::格式化输出并换行;
use crate::退出::退出;

pub use context::任务上下文;

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
    tasks: [TaskControlBlock; 3],
    /// id of current `Running` task
    current_task: usize,
}

/// Global variable: TASK_MANAGER
pub static mut TASK_MANAGER: TaskManager = TaskManager {
    num_app: 0,
    tasks: [TaskControlBlock {task_status: TaskStatus::Ready, task_cx: 任务上下文 {返回地址寄存器:0, 栈寄存器:0, 被调用者保存寄存器:[0;12]}}; 3],
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
        let next_task_cx_ptr = &task0.task_cx as *const 任务上下文;
        let mut _unused = 任务上下文::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut 任务上下文, next_task_cx_ptr);
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
            let current_task_cx_ptr = &mut self.tasks[current].task_cx as *mut 任务上下文;
            let next_task_cx_ptr = &self.tasks[next].task_cx as *const 任务上下文;
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            格式化输出并换行!("[Kernel] All applications completed!");

            退出();
        }
    }
}

/// run first task
pub fn run_first_task() {
    let num_app = 读取应用程序数目();
    let tasks = [
        TaskControlBlock {
            task_cx: 任务上下文::goto_restore(init_app_cx(0)),
            task_status: TaskStatus::Ready
        },
        TaskControlBlock {
            task_cx: 任务上下文::goto_restore(init_app_cx(1)),
            task_status: TaskStatus::Ready
        },
        TaskControlBlock {
            task_cx: 任务上下文::goto_restore(init_app_cx(2)),
            task_status: TaskStatus::Ready
        }
    ];
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
