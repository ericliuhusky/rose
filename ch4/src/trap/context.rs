#[repr(C)]
pub struct 陷入上下文 {
    // x1~x31
    pub 通用寄存器: [usize; 32],
    pub 触发异常指令地址: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}

impl 陷入上下文 {
    pub fn 应用初始上下文(应用入口地址: usize, 栈寄存器: usize,
        kernel_satp: usize,
        trap_handler: usize) -> Self {
        let mut 上下文 = Self {
            通用寄存器: [0; 32],
            触发异常指令地址: 应用入口地址,
            kernel_satp,  // addr of page table
            trap_handler, // addr of trap_handler function
        };
        上下文.通用寄存器[2] = 栈寄存器;
        上下文
    }
}
