#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate alloc;
#[macro_use]
extern crate lib;

use core::arch::asm;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

const DEFAULT_STACK_SIZE: usize = 4096; //128 got  SEGFAULT, 256(1024, 4096) got right results.
static mut RUNTIME: usize = 0;
static mut TID: usize = 0;

pub struct Runtime {
    ready_queue: VecDeque<Task>,
    current: Option<Task>,
}

struct Task {
    id: usize,
    stack: Vec<u8>,
    ctx: TaskContext,
}

#[derive(Default)]
#[repr(C)] // not strictly needed but Rust ABI is not guaranteed to be stable
pub struct TaskContext {
    ra: u64, //ra: return addres
    sp: u64, //sp
    s: [usize; 12],
    nx1: u64, //new return addres
}

impl Task {
    fn new() -> Self {
        let tid = unsafe { &mut TID };
        let task = Task {
            id: *tid,
            stack: vec![0u8; DEFAULT_STACK_SIZE],
            ctx: TaskContext::default(),
        };
        *tid += 1;
        task
    }
}

impl Runtime {
    pub fn new() -> Self {
        let base_task = Task {
            id: 0,
            stack: vec![0u8; DEFAULT_STACK_SIZE],
            ctx: TaskContext::default(),
        };

        Runtime {
            ready_queue: VecDeque::new(),
            current: Some(base_task),
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) {
        while self.yield_() {}
        println!("All tasks finished!");
    }

    fn t_return(&mut self) {
        if let Some(mut next_task) = self.ready_queue.pop_front() {
            self.current = Some(next_task);
            let old_ctx = &mut TaskContext::default();
            let new_ctx = &self.current.as_ref().unwrap().ctx;
            unsafe { switch(old_ctx, new_ctx) }
        }
    }

    fn yield_(&mut self) -> bool {
        if let Some(mut next_task) = self.ready_queue.pop_front() {
            let mut old_task = self.current.take().unwrap();
            self.current = Some(next_task);
            self.ready_queue.push_back(old_task);
            let old_ctx = &mut self.ready_queue.back_mut().unwrap().ctx;
            let new_ctx = &self.current.as_ref().unwrap().ctx;
            unsafe { switch(old_ctx, new_ctx) }
            true
        } else {
            false
        }
    }

    pub fn spawn(&mut self, f: fn()) {
        let mut task = Task::new();
        println!("RUNTIME: spawning task {}", task.id);
        let size = task.stack.len();
        unsafe {
            let s_ptr = task.stack.as_mut_ptr().offset(size as isize);
            let s_ptr = (s_ptr as usize & !7) as *mut u8;

            task.ctx.ra = guard as u64; //ctx.x1  is old return address
            task.ctx.nx1 = f as u64; //ctx.nx2 is new return address
            task.ctx.sp = s_ptr.offset(-32) as u64; //cxt.x2 is sp
        }
        self.ready_queue.push_back(task);
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    };
}

pub fn yield_task() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).yield_();
    };
}

#[naked]
extern "C" fn switch(old_ctx: &mut TaskContext, new_ctx: &TaskContext) {
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
            sd ra, 14*8(a0)
    
            ld ra, 0(a1)
            ld sp, 1*8(a1)
            ld s0, 2*8(a1)
            ld s1, 3*8(a1)
            ld s2, 4*8(a1)
            ld s3, 5*8(a1)
            ld s4, 6*8(a1)
            ld s5, 7*8(a1)
            ld s6, 8*8(a1)
            ld s7, 9*8(a1)
            ld s8, 10*8(a1)
            ld s9, 11*8(a1)
            ld s10, 12*8(a1)
            ld s11, 13*8(a1)
            ld t0, 14*8(a1)
    
            jr t0
        ",
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn main() {
    println!("stackful_coroutine begin...");
    println!("TASK  0(Runtime) STARTING");
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("TASK  1 STARTING");
        let id = 1;
        for i in 0..4 {
            println!("task: {} counter: {}", id, i);
            yield_task();
        }
        println!("TASK 1 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 2 STARTING");
        let id = 2;
        for i in 0..8 {
            println!("task: {} counter: {}", id, i);
            yield_task();
        }
        println!("TASK 2 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 3 STARTING");
        let id = 3;
        for i in 0..12 {
            println!("task: {} counter: {}", id, i);
            yield_task();
        }
        println!("TASK 3 FINISHED");
    });
    runtime.spawn(|| {
        println!("TASK 4 STARTING");
        let id = 4;
        for i in 0..16 {
            println!("task: {} counter: {}", id, i);
            yield_task();
        }
        println!("TASK 4 FINISHED");
    });
    runtime.run();
    println!("stackful_coroutine PASSED");
}
