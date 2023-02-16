//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::ops::Range;
use super::{frame_alloc, PhysPageNum, VirtPageNum};
use crate::mm::address::{page_offset};
use alloc::vec::Vec;

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new_address(ppn: PhysPageNum, is_user: bool) -> Self {
        let mut flags = 0xf;
        if is_user {
            flags |= 0x10;
        }
        PageTableEntry(ppn.0 << 10 | flags)
    }

    pub fn new_pointer(ppn: PhysPageNum) -> Self {
        PageTableEntry(ppn.0 << 10 | 0x1)
    }
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> 10)
    }
    pub fn is_valid(&self) -> bool {
        self.0 & 0x1 == 1
    }
}

/// page table structure
pub struct PageTable {
    pub root_ppn: PhysPageNum
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    pub fn new() -> Self {
        let ppn = frame_alloc();
        PageTable {
            root_ppn: ppn
        }
    }
    fn find_pte_create(&self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        for i in 0..2 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if !pte.is_valid() {
                let ppn = frame_alloc();
                *pte = PageTableEntry::new_pointer(ppn);
            }
            ppn = pte.ppn();
        }
        let pte = &mut ppn.get_pte_array()[idxs[2]];
        pte
    }
    fn find_pte(&self, vpn: VirtPageNum) -> PhysPageNum {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = ppn.get_pte_array()[idxs[i]];
            if !pte.is_valid() {
                panic!()
            }
            ppn = pte.ppn();
        }
        ppn
    }
    pub fn map(&self, vpn: VirtPageNum, ppn: PhysPageNum, is_user: bool) {
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid());
        *pte = PageTableEntry::new_address(ppn, is_user);
    }
    pub fn translate(&self, vpn: VirtPageNum) -> PhysPageNum {
        self.find_pte(vpn)
    }
    pub fn translated_byte_buffer(&self, va_range: Range<usize>) -> Vec<&'static mut [u8]> {
        let mut start = va_range.start;
        let end = va_range.end;
        let mut v = Vec::new();
        while start < end {
            let vpn = VirtPageNum::from(start);
            let ppn = self.translate(vpn);
            let next_addr = vpn.next().address();
            if next_addr <= end {
                v.push(&mut ppn.get_bytes_array()[page_offset(start)..]);
            } else {
                v.push(&mut ppn.get_bytes_array()[page_offset(start)..page_offset(end)]);
            }
            start = next_addr;
        }
        v
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
