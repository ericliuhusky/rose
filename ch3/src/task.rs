use crate::exception::Context;
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

pub struct TaskManager {
    ready_queue: VecDeque<Task>,
    current: Option<Task>,
}

impl TaskManager {
    pub fn init() {
        let 任务数目 = loader::read_app_num();
        let mut 任务列表 = VecDeque::new();
        for i in 0..任务数目 {
            let (entry_address, user_stack_top) = 加载应用到应用内存区(i);
            assert!(entry_address > unsafe { APP_START_ADDR });
            unsafe {
                let cx_ptr = CONTEXT_START_ADDRS[i] as *mut Context;
                *cx_ptr = Context::app_init(
                    entry_address,
                    user_stack_top
                );
            }
            任务列表.push_back(Task {
                i,
                status: TaskStatus::Ready
            })
        }
        unsafe {
            任务管理器 = TaskManager {
                ready_queue: 任务列表,
                current: None
            };
        }
    }

    pub fn suspend_and_run_next() {
        unsafe {
            任务管理器.current.as_mut().unwrap().status = TaskStatus::Ready;
            任务管理器.ready_queue.push_back(任务管理器.current.take().unwrap());
            Self::run_next();
        }
    }

    pub fn exit_and_run_next() {
        unsafe {
            任务管理器.current.as_mut().unwrap().status = TaskStatus::Exited;
            Self::run_next();
        }
    }


    pub fn run_next() {
        unsafe {
            if let Some(mut 下一个任务) = 任务管理器.ready_queue.pop_front() {
                下一个任务.status = TaskStatus::Running;
                let i = 下一个任务.i;
                任务管理器.current = Some(下一个任务);
                CONTEXT_START_ADDR = CONTEXT_START_ADDRS[i];
                extern "C" {
                    fn __restore();
                }
                __restore();
            } else {
                println!("[Kernel] All applications completed!");
                shutdown();
            }
        }
    }
}

static mut 任务管理器: TaskManager = TaskManager {
    ready_queue: VecDeque::new(),
    current: None
};


fn 加载应用到应用内存区(应用索引: usize) -> (usize, usize) {
    unsafe {
        let 应用数据 = loader::read_app_data(应用索引);
        let elf = elf_reader::ElfFile::read(应用数据);
        println!("{:x}", elf.entry_address());
        for p in elf.programs() {
            let start_va = p.virtual_address_range().start;
            let end_va = p.virtual_address_range().end;
            println!("{:x},{:x}", start_va, end_va);
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
        let last_p_va_end = elf.programs().last().unwrap().virtual_address_range().end;
        let user_stack_top = last_p_va_end +0x2000;
        (elf.entry_address(), user_stack_top)
    }
}
