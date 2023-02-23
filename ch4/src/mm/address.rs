use core::ops::Range;
use crate::mm::page_table::PageTableEntry;


#[derive(Clone)]
pub struct 物理页 {
    pub 页号: usize,
    pub 对齐到分页的地址范围: Range<usize>
}

pub struct 虚拟页 {
    pub 页号: usize,
    pub 对齐到分页的地址范围: Range<usize>
}


pub fn 页内偏移(v: usize) -> usize {
    v & 0xfff
}

impl 虚拟页 {
    pub fn 新建(页号: usize) -> Self {
        let 对齐到分页的起始地址 = 页号 << 12;
        let 对齐到分页的结尾地址 = (页号 + 1) << 12;
        Self { 
            页号,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址
        }
    }
}

impl 物理页 {
    pub fn 新建(页号: usize) -> Self {
        let 对齐到分页的起始地址 = 页号 << 12;
        let 对齐到分页的结尾地址 = (页号 + 1) << 12;
        Self { 
            页号,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址
        }
    }

    pub fn 读取页表项列表(&self) -> &'static mut [PageTableEntry] {
        let pa = self.对齐到分页的地址范围.start;
        unsafe {
            &mut *(pa as *mut [PageTableEntry; 512])
        }
    }
}
