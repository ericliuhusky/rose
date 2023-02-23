use core::ops::Range;

pub struct 内存地址(pub usize);

impl 内存地址 {
    pub fn 页内偏移(&self) -> usize {
        self.0 & 0xfff
    }
}

#[derive(Clone)]
pub struct 内存分页 {
    pub 页号: usize,
    pub 对齐到分页的地址范围: Range<usize>
}

impl 内存分页 {
    pub fn 新建(页号: usize) -> Self {
        let 对齐到分页的起始地址 = 页号 << 12;
        let 对齐到分页的结尾地址 = (页号 + 1) << 12;
        Self { 
            页号,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址
        }
    }
}
