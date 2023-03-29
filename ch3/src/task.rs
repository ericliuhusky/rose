use exception::{context::Context, restore::restore_context};
use alloc::collections::VecDeque;
use sbi_call::shutdown;
use crate::segment::{CONTEXT_START_ADDRS, CONTEXT_START_ADDR, APP_START_ADDR};


struct Task {
    status: TaskStatus,
    i: usize
}

#[derive(PartialEq)]
enum TaskStatus {
    Ready,
    Running,
    Exited,
}

struct TaskManager {
    ready_queue: VecDeque<Task>,
    current: Option<Task>,
}

impl TaskManager {
    fn suspend_and_run_next(&mut self) {
        self.current.as_mut().unwrap().status = TaskStatus::Ready;
        self.ready_queue.push_back(self.current.take().unwrap());
        self.run_next();
    }

    fn exit_and_run_next(&mut self) {
        self.current.as_mut().unwrap().status = TaskStatus::Exited;
        self.run_next();
    }


    fn run_next(&mut self) {
        if let Some(mut next) = self.ready_queue.pop_front() {
            next.status = TaskStatus::Running;
            unsafe {
                CONTEXT_START_ADDR = CONTEXT_START_ADDRS[*&next.i];
            }
            self.current = Some(next);
            restore_context();
        } else {
            println!("[Kernel] All applications completed!");
            shutdown();
        }
    }
}

static mut 任务管理器: TaskManager = TaskManager {
    ready_queue: VecDeque::new(),
    current: None
};

pub fn init() {
    let n = loader::read_app_num();
    let mut ready_queue = VecDeque::new();
    for i in 0..n {
        let (entry_address, user_stack_top) = 加载应用到应用内存区(i);
        assert!(entry_address > unsafe { APP_START_ADDR });
        unsafe {
            let cx_ptr = CONTEXT_START_ADDRS[i] as *mut Context;
            *cx_ptr = Context::app_init(
                entry_address,
                user_stack_top
            );
        }
        ready_queue.push_back(Task {
            i,
            status: TaskStatus::Ready
        })
    }
    unsafe {
        任务管理器 = TaskManager {
            ready_queue,
            current: None
        };
    }
}

pub fn suspend_and_run_next() {
    unsafe {
        任务管理器.suspend_and_run_next();
    }
}

pub fn exit_and_run_next() {
    unsafe {
        任务管理器.exit_and_run_next();
    }
}

pub fn run_next() {
    unsafe {
        任务管理器.run_next();
    }
}

fn 加载应用到应用内存区(应用索引: usize) -> (usize, usize) {
    unsafe {
        let 应用数据 = loader::read_app_data(应用索引);
        let elf = elf_reader::ElfFile::read(应用数据);
        println!("{:x}", elf.entry_address());
        for p in elf.programs() {
            let start_va = p.start_va();
            let end_va = p.end_va();
            if start_va < 0x80200000 {
                continue;
            }
            let dst = core::slice::from_raw_parts_mut(start_va as *mut u8, end_va - start_va);
            let src = p.data;
            let len = dst.len().min(src.len());
            for j in 0..len {
                dst[j] = src[j];
            }
        }
        let last_p_va_end = elf.programs().last().unwrap().end_va();
        let user_stack_top = last_p_va_end +0x2000;
        (elf.entry_address(), user_stack_top)
    }
}
