use core::ops::Range;
use super::address::内存地址;

/// 一段连续地址的虚拟内存
pub struct 逻辑段 {
    pub 虚拟地址范围: Range<usize>
}

impl 逻辑段 {
    pub fn 虚拟页号范围(&self) -> Range<usize> {
        let 起始页号 = 内存地址(self.虚拟地址范围.start).对齐到分页向下取整().页号();
        let 结尾页号 = 内存地址(self.虚拟地址范围.end).对齐到分页向上取整().页号();
        起始页号..结尾页号
    }

    pub fn 对齐到分页的结尾地址(&self) -> usize {
        内存地址(self.虚拟地址范围.end).对齐到分页向上取整().0
    }
}
