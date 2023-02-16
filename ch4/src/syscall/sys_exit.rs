use crate::task::exit_current_and_run_next;

pub fn sys_exit(code: i32) -> isize {
    println!("[kernel] Application exited with code {}", code);
    exit_current_and_run_next();
    -1
}
