use crate::task::TASK_MANAGER;

pub fn sys_exit(code: i32) -> isize {
    println!("[kernel] Application exited with code {}", code);
    unsafe {
        TASK_MANAGER.exit_and_run_next();
    }
    -1
}
