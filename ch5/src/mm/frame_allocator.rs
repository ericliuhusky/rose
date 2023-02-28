use alloc::vec::Vec;

use super::memory_set::可用物理内存结尾地址;
use super::address::{内存地址, 内存分页};

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

    pub fn 分配物理页并返回页号() -> 物理帧 {
        unsafe {
            if 物理内存管理器.应当分配的物理页号 == 物理内存管理器.可用物理内存结尾页号 {
                panic!()
            }
            if let Some(物理页号) = 物理内存管理器.已回收可用的物理页号列表.pop() {
                物理帧::新建(物理页号)
            } else {
                let 应当分配的物理页号 = 物理内存管理器.应当分配的物理页号;
                物理内存管理器.应当分配的物理页号 += 1;
                物理帧::新建(应当分配的物理页号)
            }
        }
    }

    fn 回收物理帧(物理页号: usize) {
        unsafe {
            物理内存管理器.已回收可用的物理页号列表.push(物理页号);
        }
    }
}

static mut 物理内存管理器: 物理内存管理器 = 物理内存管理器 {
    应当分配的物理页号: 0,
    可用物理内存结尾页号: 0,
    已回收可用的物理页号列表: Vec::new(),
};

pub struct 物理帧 {
    pub 物理页号: usize
}

impl 物理帧 {
    fn 新建(物理页号: usize) -> Self {
        // 清空物理帧
        unsafe {
            *(内存分页(物理页号).起始地址() as *mut [u8; 0x1000]) = [0; 0x1000];
        }
        Self { 物理页号 }
    }
}

impl Drop for 物理帧 {
    fn drop(&mut self) {
        物理内存管理器::回收物理帧(self.物理页号);
    }
}
