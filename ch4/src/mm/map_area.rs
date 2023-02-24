use core::ops::Range;
use alloc::vec::Vec;
use super::address::内存地址;

/// 一段连续地址的虚拟内存
pub struct 逻辑段 {
    pub 起始页号: usize,
    pub 结尾页号: usize,
    pub 结尾地址: usize
}

impl 逻辑段 {
    pub fn 新建(虚拟地址范围: Range<usize>) -> Self {
        let 结尾地址 = 内存地址(虚拟地址范围.end).对齐到分页向上取整();
        let 起始页号 = 内存地址(虚拟地址范围.start).对齐到分页向下取整().页号();
        let 结尾页号 = 结尾地址.页号();
        Self {
            起始页号,
            结尾页号,
            结尾地址: 结尾地址.0,
        }
    }
    pub fn 新建内嵌于地址范围的逻辑段(虚拟地址范围: Range<usize>) -> Self {
        let 结尾地址 = 内存地址(虚拟地址范围.end).对齐到分页向下取整();
        let 起始页号 = 内存地址(虚拟地址范围.start).对齐到分页向上取整().页号();
        let 结尾页号 = 结尾地址.页号();
        Self {
            起始页号,
            结尾页号,
            结尾地址: 结尾地址.0,
        }
    }
    pub fn 虚拟页号列表(&self) -> Vec<usize> {
        let mut 虚拟页列表 = Vec::new();
        for 虚拟页号 in self.起始页号..self.结尾页号 {
            虚拟页列表.push(虚拟页号)
        }
        虚拟页列表
    }
}

