use crate::mm::address::内存分页;
use crate::config::可用物理内存结尾地址;
use crate::mm::map_area::MapArea;

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
            let 可分配段 = MapArea::新建内嵌于地址范围的逻辑段(ekernel as usize..可用物理内存结尾地址);

            物理内存管理器 = Self {
                应当分配的物理页号: 可分配段.起始页号,
                可用物理内存结尾页号: 可分配段.结尾页号
            };
        }
    }

    pub fn 分配物理页() -> 内存分页 {
        unsafe {
            if 物理内存管理器.应当分配的物理页号 == 物理内存管理器.可用物理内存结尾页号 {
                panic!()
            }
            let 应当分配的物理页号 = 物理内存管理器.应当分配的物理页号;
            物理内存管理器.应当分配的物理页号 += 1;
            内存分页::新建(应当分配的物理页号)
        }
    }
}

static mut 物理内存管理器: 物理内存管理器 = 物理内存管理器 {
    应当分配的物理页号: 0,
    可用物理内存结尾页号: 0,
};
