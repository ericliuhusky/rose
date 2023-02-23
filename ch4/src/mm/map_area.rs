use core::ops::Range;
use alloc::vec::Vec;
use crate::mm::address::内存分页;

fn 将地址转为页号(地址: usize) -> usize {
    地址 >> 12
}
fn 对齐到分页向下取整(地址: usize) -> usize {
    地址 & !0xfff
}
fn 对齐到分页向上取整(地址: usize) -> usize {
    (地址 + 0xfff) & !0xfff
}

/// map area structure, controls a contiguous piece of virtual memory
pub struct MapArea {
    pub 起始页号: usize,
    pub 结尾页号: usize,
    pub 结尾地址: usize
}

impl MapArea {
    pub fn new(va_range: Range<usize>) -> Self {
        let 对齐到分页的结尾地址 = 对齐到分页向上取整(va_range.end);
        let start_vpn = 将地址转为页号(对齐到分页向下取整(va_range.start));
        let end_vpn = 将地址转为页号(对齐到分页的结尾地址);
        Self {
            起始页号: start_vpn,
            结尾页号: end_vpn,
            结尾地址: 对齐到分页的结尾地址,
        }
    }
    pub fn 新建内嵌于地址范围的逻辑段(va_range: Range<usize>) -> Self {
        let 对齐到分页的结尾地址 = 对齐到分页向下取整(va_range.end);
        let start_vpn = 将地址转为页号(对齐到分页向上取整(va_range.start));
        let end_vpn = 将地址转为页号(对齐到分页的结尾地址);
        Self {
            起始页号: start_vpn,
            结尾页号: end_vpn,
            结尾地址: 对齐到分页的结尾地址,
        }
    }
    pub fn 虚拟页列表(&self) -> Vec<内存分页> {
        let mut v = Vec::new();
        for vpn in self.起始页号..self.结尾页号 {
            v.push(内存分页::新建(vpn))
        }
        v
    }
}

