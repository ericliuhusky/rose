use core::ops::Range;
use crate::mm::address::内存分页;
use alloc::vec::Vec;
use crate::mm::frame_allocator::物理内存管理器;
use super::{address::{内存地址, 连续地址虚拟内存}, frame_allocator::物理帧};

#[repr(C)]
struct 页表项(usize);

impl 页表项 {
    fn 新建存放物理页号的页表项(物理页号: usize, 用户是否可见: bool) -> Self {
        let mut flags = 0xf;
        if 用户是否可见 {
            flags |= 0x10;
        }
        页表项(物理页号 << 10 | flags)
    }

    fn 新建指向下一级页表的页表项(物理页号: usize) -> Self {
        页表项(物理页号 << 10 | 0x1)
    }

    fn 物理页号(&self) -> usize {
        self.0 >> 10 
    }

    fn 是有效的(&self) -> bool {
        self.0 & 0x1 == 1
    }
}

pub struct 页表 {
    pub 物理页号: usize
}

impl 页表{
    fn 读取页表项列表(&self) -> &'static mut [页表项] {
        unsafe {
            &mut *(内存分页(self.物理页号).起始地址() as *mut [页表项; 512])
        }
    }

    fn 子页表(&self, 索引: usize) -> 页表 {
        let 目的页表项 = &self.读取页表项列表()[索引];
        if 目的页表项.是有效的() {
            Self { 物理页号: 目的页表项.物理页号() }
        } else {
            panic!()
        }
    }

    fn 子页表_没有子页表时创建(&self, 索引: usize, 物理帧列表: &mut Vec<物理帧>) -> 页表 {
        let 目的页表项 = &self.读取页表项列表()[索引];
        if 目的页表项.是有效的() {
            Self { 物理页号: 目的页表项.物理页号() }
        } else {
            let 物理帧 = 物理内存管理器::分配物理页并返回页号();
            let 物理页号 = 物理帧.物理页号;
            self.读取页表项列表()[索引] = 页表项::新建指向下一级页表的页表项(物理页号);
            物理帧列表.push(物理帧);
            Self { 物理页号 }
        }
    }
}

pub struct 多级页表 {
    pub 根页表: 页表,
    pub 物理帧列表: Vec<物理帧>,
}

impl 多级页表 {
    fn 查找存放物理页号的页表项(&self, 虚拟页号: usize) -> &mut 页表项 {
        let 一级索引 = (虚拟页号 >> 18) & 0x1ff;
        let 二级索引 = (虚拟页号 >> 9) & 0x1ff;
        let 三级索引 = 虚拟页号 & 0x1ff;
        let 一级页表 = &self.根页表;
        let 二级页表 = 一级页表.子页表(一级索引);
        let 三级页表 = 二级页表.子页表(二级索引);
        let 存放物理页号的页表项 = &mut 三级页表.读取页表项列表()[三级索引];
        存放物理页号的页表项
    }

    fn 查找存放物理页号的页表项_没有子页表时创建(&mut self, 虚拟页号: usize) -> &mut 页表项 {
        let 一级索引 = (虚拟页号 >> 18) & 0x1ff;
        let 二级索引 = (虚拟页号 >> 9) & 0x1ff;
        let 三级索引 = 虚拟页号 & 0x1ff;
        let 一级页表 = &self.根页表;
        let 二级页表 = 一级页表.子页表_没有子页表时创建(一级索引, &mut self.物理帧列表);
        let 三级页表 = 二级页表.子页表_没有子页表时创建(二级索引, &mut self.物理帧列表);
        let 存放物理页号的页表项 = &mut 三级页表.读取页表项列表()[三级索引];
        存放物理页号的页表项
    }

    pub fn 映射(&mut self, 虚拟页号: usize, 物理页号: usize, 用户是否可见: bool) {
        let 目的页表项 = self.查找存放物理页号的页表项_没有子页表时创建(虚拟页号);
        assert!(!目的页表项.是有效的());
        *目的页表项 = 页表项::新建存放物理页号的页表项(物理页号, 用户是否可见);
    }

    pub fn 虚拟地址范围转换物理地址范围列表(&self, 虚拟地址范围: Range<usize>) -> Vec<Range<usize>> {
        let 起始虚拟地址 = 虚拟地址范围.start;
        let 结尾虚拟地址 = 虚拟地址范围.end;
        let 虚拟页号范围 = 连续地址虚拟内存 { 虚拟地址范围 }.虚拟页号范围();
        let 虚拟页号数目 = 虚拟页号范围.len();
        虚拟页号范围
            // 虚拟页号范围转换物理页号列表
            .map(|虚拟页号| {
                self.查找存放物理页号的页表项(虚拟页号).物理页号()
            })
            // 物理页号列表转物理地址范围列表
            .enumerate()
            .map(|(i, 物理页号)| {
                let 起始物理地址;
                if i == 0 {
                    起始物理地址 = 内存分页(物理页号).起始地址() + 内存地址(起始虚拟地址).页内偏移();
                } else {
                    起始物理地址 = 内存分页(物理页号).起始地址();
                }
                let 起始结尾地址;
                if i == 虚拟页号数目 - 1 {
                    起始结尾地址 = 内存分页(物理页号).起始地址() + 内存地址(结尾虚拟地址).页内偏移();
                } else {
                    起始结尾地址 = 内存分页(物理页号).结尾地址();
                }
                起始物理地址..起始结尾地址
            })
            .collect()
    }

    fn 虚拟地址范围转换字节数组列表(&self, 虚拟地址范围: Range<usize>) -> Vec<&'static mut [u8]> {        
        let 物理地址范围列表 = self.虚拟地址范围转换物理地址范围列表(虚拟地址范围);
        物理地址范围列表
            .iter()
            .map(|物理地址范围| {
                unsafe {
                    core::slice::from_raw_parts_mut(物理地址范围.start as *mut u8, 物理地址范围.len())
                }
            })
            .collect()
    }

    pub fn 读取字节数组(&self, 虚拟地址范围: Range<usize>) -> Vec<u8> {
        let 字节数组列表 = self.虚拟地址范围转换字节数组列表(虚拟地址范围);
        let mut v = Vec::new();
        for 字节数组 in 字节数组列表 {
            for 字节 in 字节数组 {
                v.push(字节.clone());
            }
        }
        v
    }

    pub fn 写入字节数组(&self, 虚拟地址范围: Range<usize>, 数据: &[u8]) {
        let 字节数组列表 = self.虚拟地址范围转换字节数组列表(虚拟地址范围);
        let mut i = 0;
        for 字节数组 in 字节数组列表 {
            if i >= 数据.len() {
                break;
            }
            let 要写入的数据 = &数据[i..i + 字节数组.len()];
            i += 字节数组.len();
            for j in 0..字节数组.len() {
                字节数组[j] = 要写入的数据[j];
            }
        }
    }
}
