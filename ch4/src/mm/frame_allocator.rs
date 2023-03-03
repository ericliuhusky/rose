use page_table::{FrameAlloc, PPN};
use super::memory_set::可用物理内存结尾地址;
use super::address::内存地址;

pub struct 物理内存管理器 {
    应当分配的物理页号: usize,
    可用物理内存结尾页号: usize,
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
                可用物理内存结尾页号
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
            let 应当分配的物理页号 = 物理内存管理器.应当分配的物理页号;
            物理内存管理器.应当分配的物理页号 += 1;
            PPN::new(应当分配的物理页号)
        }
    }

    fn dealloc(_frame: PPN) {
        
    }
}

static mut 物理内存管理器: 物理内存管理器 = 物理内存管理器 {
    应当分配的物理页号: 0,
    可用物理内存结尾页号: 0,
};
