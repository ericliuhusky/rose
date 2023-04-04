use core::ops::Range;
use page_table::VA;

pub struct 逻辑段 {
    pub 虚拟地址范围: Range<usize>,
    pub 恒等映射: bool,
    pub 用户可见: bool
}

impl 逻辑段 {
    pub fn 虚拟页号范围(&self) -> Range<usize> {
        let 起始页号 = VA::new(self.虚拟地址范围.start).align_to_lower().page_number().0;
        let 结尾页号 = VA::new(self.虚拟地址范围.end).align_to_upper().page_number().0;
        起始页号..结尾页号
    }
}
