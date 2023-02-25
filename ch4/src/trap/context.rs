use crate::mm::page_table::多级页表;
use super::应用陷入上下文存放地址;

#[repr(C)]
pub struct 陷入上下文 {
    // x1~x31
    pub 通用寄存器: [usize; 32],
    pub 触发异常指令地址: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
}

impl 陷入上下文 {
    pub fn 应用初始上下文(应用入口地址: usize, 栈寄存器: usize,
        kernel_satp: usize) -> Self {
        let mut 上下文 = Self {
            通用寄存器: [0; 32],
            触发异常指令地址: 应用入口地址,
            kernel_satp,  // addr of page table
        };
        上下文.通用寄存器[2] = 栈寄存器;
        上下文
    }

    pub fn 应用地址空间的上下文(多级页表: &多级页表) -> &'static mut Self {
        let pa_ranges = 多级页表.虚拟地址范围转换物理地址范围列表(应用陷入上下文存放地址()..0xfffffffffffff000);
        unsafe {
            &mut *(pa_ranges[0].start as *mut 陷入上下文)
        }
    }
}
