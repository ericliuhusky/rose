pub struct 内存地址(pub usize);

impl 内存地址 {
    pub fn 页号(&self) -> usize {
        self.0 >> 12
    }

    pub fn 对齐到分页向下取整(&self) -> 内存地址 {
        内存地址(self.0 & !0xfff)
    }
    pub fn 对齐到分页向上取整(&self) -> 内存地址 {
        内存地址((self.0 + 0xfff) & !0xfff)
    }
}

use core::ops::Range;

pub struct 连续地址虚拟内存 {
    pub 虚拟地址范围: Range<usize>,
}

impl 连续地址虚拟内存 {
    pub fn 虚拟页号范围(&self) -> Range<usize> {
        let 起始页号 = 内存地址(self.虚拟地址范围.start).对齐到分页向下取整().页号();
        let 结尾页号 = 内存地址(self.虚拟地址范围.end).对齐到分页向上取整().页号();
        起始页号..结尾页号
    }

    pub fn 对齐到分页的结尾地址(&self) -> usize {
        内存地址(self.虚拟地址范围.end).对齐到分页向上取整().0
    }
}

pub struct 逻辑段 {
    pub 连续地址虚拟内存: 连续地址虚拟内存,
    pub 恒等映射: bool,
    pub 用户可见: bool
}
