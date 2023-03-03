use alloc::vec::Vec;
use crate::page_table::{FrameAlloc, PPN};

use super::memory_set::可用物理内存结尾地址;
use super::address::内存地址;

pub struct 物理内存管理器 {
    应当分配的物理页号: usize,
    可用物理内存结尾页号: usize,
    已回收可用的物理页号列表: Vec<usize>,
}

impl 物理内存管理器 {
    pub fn 初始化() {
        extern "C" {
            // 内核结尾地址
            fn ekernel();
        }
        unsafe {
            let 应当分配的物理页号 = 内存地址(ekernel as usize).对齐到分页向上取整().页号();
            let 可用物理内存结尾页号 = 内存地址(可用物理内存结尾地址).对齐到分页向下取整().页号();
            物理内存管理器 = Self {
                应当分配的物理页号,
                可用物理内存结尾页号,
                已回收可用的物理页号列表: Vec::new(),
            };
        }
    }
}

impl FrameAlloc for 物理内存管理器 {
    fn alloc() -> PPN {
        unsafe {
            if 物理内存管理器.应当分配的物理页号 == 物理内存管理器.可用物理内存结尾页号 {
                panic!()
            }
            if let Some(物理页号) = 物理内存管理器.已回收可用的物理页号列表.pop() {
                PPN::new(物理页号)
            } else {
                let 应当分配的物理页号 = 物理内存管理器.应当分配的物理页号;
                物理内存管理器.应当分配的物理页号 += 1;
                PPN::new(应当分配的物理页号)
            }
        }
    }

    fn dealloc(frame: PPN) {
        unsafe {
            *(frame.start_addr().0 as *mut [u8; 0x1000]) = [0; 0x1000];
            物理内存管理器.已回收可用的物理页号列表.push(frame.0);
        }
    }
}

static mut 物理内存管理器: 物理内存管理器 = 物理内存管理器 {
    应当分配的物理页号: 0,
    可用物理内存结尾页号: 0,
    已回收可用的物理页号列表: Vec::new(),
};

