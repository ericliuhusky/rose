#![no_std]
#![no_main]
#![feature(naked_functions)]

#[macro_use]
extern crate lib;

mod user_thread {
    extern crate alloc;

    use alloc::collections::VecDeque;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::arch::asm;
    use core::mem::MaybeUninit;

    #[repr(C)]
    struct TaskContext {
        ra: usize,
        sp: usize,
        s: [usize; 12],
    }

    struct Task {
        stack: Vec<u8>,
        cx: TaskContext,
    }

    impl Task {
        fn new() -> Self {
            Self {
                stack: vec![0u8; 0x1000],
                cx: TaskContext {
                    ra: 0,
                    sp: 0,
                    s: [0; 12],
                },
            }
        }

        fn stack_top(&self) -> usize {
            self.stack.as_ptr() as usize + self.stack.len()
        }
    }

    struct TaskManager {
        ready_queue: VecDeque<Task>,
        current: Option<Task>,
    }

    impl TaskManager {
        pub fn new() -> Self {
            let main_task = Task::new();

            Self {
                ready_queue: VecDeque::new(),
                current: Some(main_task),
            }
        }

        fn suspend_and_run_next(&mut self) {
            let task = self.current.take().unwrap();
            self.ready_queue.push_back(task);
            self.run_next();
        }

        fn exit_and_run_next(&mut self) {
            self.current.take();
            self.run_next();
        }

        fn run_next(&mut self) {
            if let Some(next) = self.ready_queue.pop_front() {
                self.current = Some(next);
                let cx = &self.current.as_ref().unwrap().cx;
                restore_context(cx);
            }
        }

        fn add_task(&mut self, f: fn()) {
            let mut task = Task::new();
            task.cx.ra = f as usize;
            task.cx.sp = task.stack_top();
            self.ready_queue.push_back(task);
        }

        fn run(&self) {
            loop {
                if self.ready_queue.is_empty() {
                    break;
                }
                yield_();
            }
        }
    }

    static mut TASK_MANAGER: MaybeUninit<TaskManager> = MaybeUninit::uninit();

    pub fn init() {
        unsafe {
            TASK_MANAGER.write(TaskManager::new());
        }
    }

    static mut FUNC_ID: usize = 0;
    const FUNC_YIELD: usize = 0;
    const FUNC_EXIT: usize = 1;

    pub fn exit() {
        unsafe {
            FUNC_ID = FUNC_EXIT;
            let current = (*TASK_MANAGER.as_mut_ptr()).current.as_mut().unwrap();
            save_context(&mut current.cx);
        }
    }

    pub fn yield_() {
        unsafe {
            FUNC_ID = FUNC_YIELD;
            let current = (*TASK_MANAGER.as_mut_ptr()).current.as_mut().unwrap();
            save_context(&mut current.cx);
        }
    }

    pub fn spawn(f: fn()) {
        unsafe {
            (*TASK_MANAGER.as_mut_ptr()).add_task(f);
        }
    }

    pub fn run() {
        unsafe {
            (*TASK_MANAGER.as_ptr()).run();
        }
    }

    #[no_mangle]
    fn user_thread_handler() {
        unsafe {
            if FUNC_ID == FUNC_YIELD {
                (*TASK_MANAGER.as_mut_ptr()).suspend_and_run_next();
            } else if FUNC_ID == FUNC_EXIT {
                (*TASK_MANAGER.as_mut_ptr()).exit_and_run_next();
            }
        }
    }

    #[naked]
    extern "C" fn save_context(cx: &mut TaskContext) {
        unsafe {
            asm!(
                "
            sd ra, 0(a0)
            sd sp, 1*8(a0)
            sd s0, 2*8(a0)
            sd s1, 3*8(a0)
            sd s2, 4*8(a0)
            sd s3, 5*8(a0)
            sd s4, 6*8(a0)
            sd s5, 7*8(a0)
            sd s6, 8*8(a0)
            sd s7, 9*8(a0)
            sd s8, 10*8(a0)
            sd s9, 11*8(a0)
            sd s10, 12*8(a0)
            sd s11, 13*8(a0)

            call user_thread_handler
        ",
                options(noreturn)
            );
        }
    }

    #[naked]
    extern "C" fn restore_context(cx: &TaskContext) {
        unsafe {
            asm!(
                "
            ld ra, 0(a0)
            ld sp, 1*8(a0)
            ld s0, 2*8(a0)
            ld s1, 3*8(a0)
            ld s2, 4*8(a0)
            ld s3, 5*8(a0)
            ld s4, 6*8(a0)
            ld s5, 7*8(a0)
            ld s6, 8*8(a0)
            ld s7, 9*8(a0)
            ld s8, 10*8(a0)
            ld s9, 11*8(a0)
            ld s10, 12*8(a0)
            ld s11, 13*8(a0)

            ret
        ",
                options(noreturn)
            );
        }
    }
}

#[no_mangle]
fn main() {
    user_thread::init();
    user_thread::spawn(|| {
        for i in 0..10 {
            println!("A {}", i);
            user_thread::yield_();
        }
        user_thread::exit();
    });
    user_thread::spawn(|| {
        for i in 0..10 {
            println!("B {}", i);
            user_thread::yield_();
        }
        user_thread::exit();
    });
    user_thread::spawn(|| {
        for i in 0..10 {
            println!("C {}", i);
            user_thread::yield_();
        }
        user_thread::exit();
    });
    user_thread::run();
    println!("All tasks completed!");
}
