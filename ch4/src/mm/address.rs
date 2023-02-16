//! Implementation of physical and virtual address and page number.

use super::PageTableEntry;
use crate::config::PAGE_SIZE_BITS;


/// physical page number
#[derive(Copy, Clone)]
pub struct PhysPageNum(pub usize);

/// virtual page number
#[derive(Copy, Clone)]
pub struct VirtPageNum(pub usize);



pub fn floor(v: usize) -> usize {
    v >> PAGE_SIZE_BITS
}
pub fn ceil(v: usize) -> usize {
    (v + (1 << PAGE_SIZE_BITS) - 1) >> PAGE_SIZE_BITS
}
pub fn page_offset(v: usize) -> usize {
    v & 0xfff
}

impl VirtPageNum {
    pub fn from(address: usize) -> Self {
        VirtPageNum(floor(address))
    }

    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

    pub fn address(&self) -> usize {
        self.0 << PAGE_SIZE_BITS
    }

    pub fn next(&self) -> Self {
        VirtPageNum(self.0 + 1)
    }
}

impl PhysPageNum {
    pub fn from(address: usize) -> Self {
        PhysPageNum(floor(address))
    }

    pub fn address(&self) -> usize {
        self.0 << PAGE_SIZE_BITS
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa = self.address();
        unsafe { core::slice::from_raw_parts_mut(pa as *mut PageTableEntry, 512) }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa = self.address();
        unsafe { core::slice::from_raw_parts_mut(pa as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa = self.address();
        unsafe { (pa as *mut T).as_mut().unwrap() }
    }
}
