use crate::batch::run_next_app;

pub fn sys_exit(code: i32) -> isize {
    println!("[kernel] Application exited with code {}", code);
    run_next_app();
    -1
}
