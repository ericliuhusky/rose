use core::ops::Range;
use crate::mm::address::内存分页;
use alloc::vec::Vec;
use crate::mm::frame_allocator::物理内存管理器;
use super::address::{内存地址, 逻辑段};

#[repr(C)]
pub struct 页表项(usize);

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

    fn 子页表(&self, 索引: usize, 没有子页表时创建: bool) -> 页表 {
        let pte = &self.读取页表项列表()[索引];
        if pte.是有效的() {
            Self { 物理页号: pte.物理页号() }
        } else {
            if 没有子页表时创建 {
                let 物理页号 = 物理内存管理器::分配物理页并返回页号();
                self.读取页表项列表()[索引] = 页表项::新建指向下一级页表的页表项(物理页号);
                Self { 物理页号 }
            } else {
                panic!()
            }
        }
    }
}

pub struct 多级页表 {
    pub 根页表: 页表
}

impl 多级页表 {
    fn 查找存放物理页号的页表项(&self, 虚拟页号: usize, 没有子页表时创建: bool) -> &mut 页表项 {
        let 一级索引 = (虚拟页号 >> 18) & 0x1ff;
        let 二级索引 = (虚拟页号 >> 9) & 0x1ff;
        let 三级索引 = 虚拟页号 & 0x1ff;
        let 一级页表 = &self.根页表;
        let 二级页表 = 一级页表.子页表(一级索引, 没有子页表时创建);
        let 三级页表 = 二级页表.子页表(二级索引, 没有子页表时创建);
        let 存放物理页号的页表项 = &mut 三级页表.读取页表项列表()[三级索引];
        存放物理页号的页表项
    }
    pub fn 映射(&self, 虚拟页号: usize, 物理页号: usize, 用户是否可见: bool) {
        let pte = self.查找存放物理页号的页表项(虚拟页号, true);
        assert!(!pte.是有效的());
        *pte = 页表项::新建存放物理页号的页表项(物理页号, 用户是否可见);
    }
    pub fn write(&self, va_range: Range<usize>, data: &[u8]) {
        let dsts = self.虚拟地址范围转换字节串列表(va_range);
        let mut i = 0;
        for dst in dsts {
            if i >= data.len() {
                break;
            }
            let src = &data[i..i + dst.len()];
            i += dst.len();
            for i in 0..dst.len() {
                dst[i] = src[i];
            }
        }
    }
    pub fn read(&self, va_range: Range<usize>) -> Vec<u8> {
        let bytes_array = self.虚拟地址范围转换字节串列表(va_range);
        let mut v = Vec::new();
        for bytes in bytes_array {
            for byte in bytes {
                v.push(byte.clone());
            }
        }
        v
    }
    fn 虚拟地址范围转换字节串列表(&self, 虚拟地址范围: Range<usize>) -> Vec<&'static mut [u8]> {        
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
    pub fn 虚拟地址范围转换物理地址范围列表(&self, 虚拟地址范围: Range<usize>) -> Vec<Range<usize>> {
        let va_start = 虚拟地址范围.start;
        let va_end = 虚拟地址范围.end;
        let vpn_range = 逻辑段 { 虚拟地址范围 }.虚拟页号范围();
        let vpn_len = vpn_range.len();
        vpn_range
            // 虚拟页号范围转换物理页号列表
            .map(|虚拟页号| {
                self.查找存放物理页号的页表项(虚拟页号, false).物理页号()
            })
            // 物理页列表转物理地址列表
            .enumerate()
            .map(|(i, pn)| {
                let pa_start;
                if i == 0 {
                    pa_start = 内存分页(pn).起始地址() + 内存地址(va_start).页内偏移();
                } else {
                    pa_start = 内存分页(pn).起始地址();
                }
                let pa_end;
                if i == vpn_len - 1 {
                    pa_end = 内存分页(pn).起始地址() + 内存地址(va_end).页内偏移();
                } else {
                    pa_end = 内存分页(pn).结尾地址();
                }
                pa_start..pa_end
            })
            .collect()
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.根页表.物理页号
    }
}
