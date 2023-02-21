//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::ops::Range;
use crate::mm::address::{页内偏移, 物理页, 虚拟页};
use alloc::vec::Vec;
use crate::config::TRAP_CONTEXT;
use crate::trap::陷入上下文;
use crate::mm::frame_allocator::FrameAllocator;

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
        let ppn = FrameAllocator::frame_alloc();
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
                let ppn = FrameAllocator::frame_alloc();
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
        let mut start = va_range.start;
        let end = va_range.end;
        let mut v = Vec::new();
        while start < end {
            let vpn = 虚拟页::地址所在的虚拟页(start);
            let ppn = self.translate(vpn);
            let next_addr = vpn.下一页().起始地址();
            if next_addr <= end {
                v.push(&mut ppn.读取字节列表()[页内偏移(start)..]);
            } else {
                v.push(&mut ppn.读取字节列表()[页内偏移(start)..页内偏移(end)]);
            }
            start = next_addr;
        }
        v
    }
    pub fn translated_trap_context(&self) -> &mut 陷入上下文 {
        let trap_cx_ppn = self.translate(虚拟页::地址所在的虚拟页(TRAP_CONTEXT));
        let trap_cx = trap_cx_ppn.以某种类型来读取();
        trap_cx
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
