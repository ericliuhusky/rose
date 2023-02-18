//! Implementation of [`TaskContext`]

/// Task Context
#[derive(Copy, Clone)]
#[repr(C)]
pub struct 任务上下文 {
    pub 返回地址寄存器: usize,
    pub 栈寄存器: usize,
    // s0~s11
    pub 被调用者保存寄存器: [usize; 12],
}

impl 任务上下文 {
    /// init task context
    pub fn zero_init() -> Self {
        Self {
            返回地址寄存器: 0,
            栈寄存器: 0,
            被调用者保存寄存器: [0; 12],
        }
    }

    /// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            返回地址寄存器: __restore as usize,
            栈寄存器: kstack_ptr,
            被调用者保存寄存器: [0; 12],
        }
    }
}
