use core::ops::Range;
use crate::mm::address::内存分页;
use alloc::vec::Vec;
use crate::config::{TRAP_CONTEXT, TRAP_CONTEXT_END};
use crate::trap::陷入上下文;
use crate::mm::frame_allocator::物理内存管理器;
use super::address::内存地址;
use super::map_area::逻辑段;

#[repr(C)]
pub struct 页表项(usize);

impl 页表项 {
    fn 新建存放物理页号的页表项(物理页: 内存分页, 用户是否可见: bool) -> Self {
        let mut flags = 0xf;
        if 用户是否可见 {
            flags |= 0x10;
        }
        页表项(物理页.页号 << 10 | flags)
    }

    fn 新建指向下一级页表的页表项(物理页: &内存分页) -> Self {
        页表项(物理页.页号 << 10 | 0x1)
    }
    fn 物理页(&self) -> 内存分页 {
        内存分页::新建(self.0 >> 10)
    }
    fn 是有效的(&self) -> bool {
        self.0 & 0x1 == 1
    }
}

#[derive(Clone)]
pub struct 页表 {
    pub 物理页: 内存分页
}

impl 页表{
    fn 新建(物理页: 内存分页) -> Self {
        Self { 物理页 }
    }

    fn 读取页表项列表(&self) -> &'static mut [页表项] {
        unsafe {
            &mut *(self.物理页.起始地址 as *mut [页表项; 512])
        }
    }

    fn 子页表(&self, 索引: usize) -> Option<页表> {
        let pte = &self.读取页表项列表()[索引];
        if pte.是有效的() {
            Some(Self::新建(pte.物理页()))
        } else {
            None
        }
    }

    fn 添加子页表(&self, 索引: usize) -> 页表 {
        let ppn = 物理内存管理器::分配物理页();
        self.读取页表项列表()[索引] = 页表项::新建指向下一级页表的页表项(&ppn);
        Self::新建(ppn)
    }
}

pub struct 多级页表 {
    pub 根页表: 页表
}

impl 多级页表 {
    pub fn 新建() -> Self {
        let 物理页 = 物理内存管理器::分配物理页();
        多级页表 {
            根页表: 页表::新建(物理页)
        }
    }

    fn find_pte_create(&self, vpn: 内存分页) -> &mut 页表项 {
        let idxs = vpn.页表项索引列表();
        let mut pt = self.根页表.clone();
        for i in 0..2 {
            if let Some(npt) = pt.子页表(idxs[i]) {
                pt = npt;
            } else {
                pt = pt.添加子页表(idxs[i]);

            }
        }
        let pte = &mut pt.读取页表项列表()[idxs[2]];
        pte
    }
    fn find_pte(&self, vpn: &内存分页) -> 内存分页 {
        let idxs = vpn.页表项索引列表();
        let mut pt = self.根页表.clone();
        for i in 0..2 {
            if let Some(npt) = pt.子页表(idxs[i]) {
                pt = npt;
            } else {
                panic!()
            }
        }
        let ppn = pt.读取页表项列表()[idxs[2]].物理页();
        ppn
    }
    pub fn 映射(&self, 虚拟页: 内存分页, 物理页: 内存分页, 用户是否可见: bool) {
        let pte = self.find_pte_create(虚拟页);
        assert!(!pte.是有效的());
        *pte = 页表项::新建存放物理页号的页表项(物理页, 用户是否可见);
    }
    fn 虚拟页转换物理页(&self, 虚拟页: &内存分页) -> 内存分页 {
        self.find_pte(虚拟页)
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
    fn 虚拟地址范围转换物理地址范围列表(&self, 虚拟地址范围: Range<usize>) -> Vec<Range<usize>> {
        let va_start = 虚拟地址范围.start;
        let va_end = 虚拟地址范围.end;
        let vp_list = 逻辑段::新建(虚拟地址范围).虚拟页列表();
        vp_list
            .iter()
            // 虚拟页列表转物理页列表
            .map(|vp| {
                self.虚拟页转换物理页(vp)
            })
            // 物理页列表转物理地址列表
            .enumerate()
            .map(|(i, pn)| {
                let pa_start;
                if i == 0 {
                    pa_start = pn.起始地址 + 内存地址(va_start).页内偏移();
                } else {
                    pa_start = pn.起始地址;
                }
                let pa_end;
                if i == vp_list.len() - 1 {
                    pa_end = pn.起始地址 + 内存地址(va_end).页内偏移();
                } else {
                    pa_end = pn.结尾地址;
                }
                pa_start..pa_end
            })
            .collect()
    }
    pub fn translated_trap_context(&self) -> &mut 陷入上下文 {
        let pa_ranges = self.虚拟地址范围转换物理地址范围列表(TRAP_CONTEXT..TRAP_CONTEXT_END);
        unsafe {
            &mut *(pa_ranges[0].start as *mut 陷入上下文)
        }
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.根页表.物理页.页号
    }
}
