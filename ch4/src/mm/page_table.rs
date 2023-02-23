//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::ops::Range;
use crate::mm::address::内存分页;
use alloc::vec::Vec;
use crate::config::{TRAP_CONTEXT, TRAP_CONTEXT_END};
use crate::trap::陷入上下文;
use crate::mm::frame_allocator::物理内存管理器;
use super::address::内存地址;
use super::map_area::MapArea;

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new_address(ppn: 内存分页, is_user: bool) -> Self {
        let mut flags = 0xf;
        if is_user {
            flags |= 0x10;
        }
        PageTableEntry(ppn.页号 << 10 | flags)
    }

    pub fn new_pointer(ppn: 内存分页) -> Self {
        PageTableEntry(ppn.页号 << 10 | 0x1)
    }
    pub fn ppn(&self) -> 内存分页 {
        内存分页::新建(self.0 >> 10)
    }
    pub fn is_valid(&self) -> bool {
        self.0 & 0x1 == 1
    }
}

/// page table structure
pub struct PageTable {
    pub root_ppn: 内存分页
}

fn 页表项索引列表(页号: usize) -> [usize; 3] {
    let mut vpn = 页号;
    let mut idx = [0usize; 3];
    for i in (0..3).rev() {
        idx[i] = vpn & 511;
        vpn >>= 9;
    }
    idx
}

fn 读取页表项列表(地址: usize) -> &'static mut [PageTableEntry] {
    unsafe {
        &mut *(地址 as *mut [PageTableEntry; 512])
    }
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    pub fn new() -> Self {
        let ppn = 物理内存管理器::分配物理页();
        PageTable {
            root_ppn: ppn
        }
    }
    fn find_pte_create(&self, vpn: 内存分页) -> &mut PageTableEntry {
        let idxs = 页表项索引列表(vpn.页号);
        let mut ppn = self.root_ppn.clone();
        for i in 0..2 {
            let pte = &mut 读取页表项列表(ppn.对齐到分页的地址范围.start)[idxs[i]];
            if !pte.is_valid() {
                let ppn = 物理内存管理器::分配物理页();
                *pte = PageTableEntry::new_pointer(ppn);
            }
            ppn = pte.ppn();
        }
        let pte = &mut 读取页表项列表(ppn.对齐到分页的地址范围.start)[idxs[2]];
        pte
    }
    fn find_pte(&self, vpn: &内存分页) -> 内存分页 {
        let idxs = 页表项索引列表(vpn.页号);
        let mut ppn = self.root_ppn.clone();
        for i in 0..3 {
            let pte = 读取页表项列表(ppn.对齐到分页的地址范围.start)[idxs[i]];
            if !pte.is_valid() {
                panic!()
            }
            ppn = pte.ppn();
        }
        ppn
    }
    pub fn map(&self, vpn: 内存分页, ppn: 内存分页, is_user: bool) {
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid());
        *pte = PageTableEntry::new_address(ppn, is_user);
    }
    pub fn translate(&self, vpn: &内存分页) -> 内存分页 {
        self.find_pte(vpn)
    }
    pub fn write(&self, va_range: Range<usize>, data: &[u8]) {
        let dsts = self.translated_byte_buffer(va_range);
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
        let bytes_array = self.translated_byte_buffer(va_range);
        let mut v = Vec::new();
        for bytes in bytes_array {
            for byte in bytes {
                v.push(byte.clone());
            }
        }
        v
    }
    fn translated_byte_buffer(&self, va_range: Range<usize>) -> Vec<&'static mut [u8]> {        
        let pa_ranges = self.translated_address(va_range);
        pa_ranges
            .iter()
            .map(|pa_range| {
                unsafe {
                    core::slice::from_raw_parts_mut(pa_range.start as *mut u8, pa_range.len())
                }
            })
            .collect()
    }
    fn translated_address(&self, va_range: Range<usize>) -> Vec<Range<usize>> {
        let va_start = va_range.start;
        let va_end = va_range.end;
        let vp_list = MapArea::new(va_range).vp_list();
        vp_list
            .iter()
            // 虚拟页列表转物理页列表
            .map(|vp| {
                self.translate(vp)
            })
            // 物理页列表转物理地址列表
            .enumerate()
            .map(|(i, pn)| {
                let pa_start;
                if i == 0 {
                    pa_start = pn.对齐到分页的地址范围.start + 内存地址(va_start).页内偏移();
                } else {
                    pa_start = pn.对齐到分页的地址范围.start;
                }
                let pa_end;
                if i == vp_list.len() - 1 {
                    pa_end = pn.对齐到分页的地址范围.start + 内存地址(va_end).页内偏移();
                } else {
                    pa_end = pn.对齐到分页的地址范围.end;
                }
                pa_start..pa_end
            })
            .collect()
    }
    pub fn translated_trap_context(&self) -> &mut 陷入上下文 {
        let pa_ranges = self.translated_address(TRAP_CONTEXT..TRAP_CONTEXT_END);
        unsafe {
            &mut *(pa_ranges[0].start as *mut 陷入上下文)
        }
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.页号
    }
}
