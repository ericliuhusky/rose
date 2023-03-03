use page_table::{FrameAlloc, PPN, PA};
use super::memory_set::可用物理内存结尾地址;

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
            let 应当分配的物理页号 = PA::new(ekernel as usize).align_to_upper().page_number().0;
            let 可用物理内存结尾页号 = PA::new(可用物理内存结尾地址).align_to_lower().page_number().0;
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
