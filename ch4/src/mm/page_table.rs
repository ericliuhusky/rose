//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::ops::Range;
use crate::mm::address::{页内偏移, 物理页, 虚拟页};
use alloc::vec::Vec;
use crate::config::{TRAP_CONTEXT, TRAP_CONTEXT_END};
use crate::trap::陷入上下文;
use crate::mm::frame_allocator::物理内存管理器;
use super::address::{将地址转为页号并向下取整, 将地址转为页号并向上取整};
use super::memory_set::MapArea;

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new_address(ppn: 物理页, is_user: bool) -> Self {
        let mut flags = 0xf;
        if is_user {
            flags |= 0x10;
        }
        PageTableEntry(ppn.0 << 10 | flags)
    }

    pub fn new_pointer(ppn: 物理页) -> Self {
        PageTableEntry(ppn.0 << 10 | 0x1)
    }
    pub fn ppn(&self) -> 物理页 {
        物理页(self.0 >> 10)
    }
    pub fn is_valid(&self) -> bool {
        self.0 & 0x1 == 1
    }
}

/// page table structure
pub struct PageTable {
    pub root_ppn: 物理页
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    pub fn new() -> Self {
        let ppn = 物理内存管理器::分配物理页();
        PageTable {
            root_ppn: ppn
        }
    }
    fn find_pte_create(&self, vpn: 虚拟页) -> &mut PageTableEntry {
        let idxs = vpn.页表项索引列表();
        let mut ppn = self.root_ppn;
        for i in 0..2 {
            let pte = &mut ppn.读取页表项列表()[idxs[i]];
            if !pte.is_valid() {
                let ppn = 物理内存管理器::分配物理页();
                *pte = PageTableEntry::new_pointer(ppn);
            }
            ppn = pte.ppn();
        }
        let pte = &mut ppn.读取页表项列表()[idxs[2]];
        pte
    }
    fn find_pte(&self, vpn: 虚拟页) -> 物理页 {
        let idxs = vpn.页表项索引列表();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = ppn.读取页表项列表()[idxs[i]];
            if !pte.is_valid() {
                panic!()
            }
            ppn = pte.ppn();
        }
        ppn
    }
    pub fn map(&self, vpn: 虚拟页, ppn: 物理页, is_user: bool) {
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid());
        *pte = PageTableEntry::new_address(ppn, is_user);
    }
    pub fn translate(&self, vpn: 虚拟页) -> 物理页 {
        self.find_pte(vpn)
    }
    pub fn translated_byte_buffer(&self, va_range: Range<usize>) -> Vec<&'static mut [u8]> {        
        let mut v = Vec::new();
        let pa_ranges = self.translated_address(va_range);
        for pa_range in pa_ranges {
            let bytes = unsafe {
                core::slice::from_raw_parts_mut(pa_range.start as *mut u8, pa_range.len())
            };
            v.push(bytes);
        }
        v
    }
    fn translated_page(&self, va_range: Range<usize>) -> Vec<物理页> {
        let vpn_range = MapArea::new(va_range).vpn_range;
        let mut ppns = Vec::new();
        for vpn in vpn_range {
            let vpn = 虚拟页(vpn);
            let ppn = self.translate(vpn);
            ppns.push(ppn);
        }
        ppns
    }
    fn translated_address(&self, va_range: Range<usize>) -> Vec<Range<usize>> {
        let va_start = va_range.start;
        let va_end = va_range.end;
        let ppns = self.translated_page(va_range);
        let mut pa_ranges = Vec::new();
        for i in 0..ppns.len() {
            let pa_start;
            if i == 0 {
                pa_start = ppns[i].起始地址() + 页内偏移(va_start);
            } else {
                pa_start = ppns[i].起始地址();
            }
            let pa_end;
            if i == ppns.len() - 1 {
                pa_end = ppns[i].起始地址() + 页内偏移(va_end);
            } else {
                pa_end = ppns[i].结尾地址();
            }
            pa_ranges.push(pa_start..pa_end);
        }
        pa_ranges
    }
    pub fn translated_trap_context(&self) -> &mut 陷入上下文 {
        let pa_ranges = self.translated_address(TRAP_CONTEXT..TRAP_CONTEXT_END);
        unsafe {
            &mut *(pa_ranges[0].start as *mut 陷入上下文)
        }
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
