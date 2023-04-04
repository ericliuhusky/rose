use super::address::{PPN, PA};
use core::ops::{Index, IndexMut};

pub struct PageTableEntryFlags(u8);

impl PageTableEntryFlags {
    pub const POINT_TO_NEXT: Self = Self(0b_0_000_1);
    pub const XWR: Self = Self(0b_0_111_1);
    pub const UXWR: Self = Self(0b_1_111_1);
}

#[repr(C)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new(ppn: PPN, flags: PageTableEntryFlags) -> Self {
        Self(ppn.0 << 10 | flags.0 as usize)
    }

    pub fn ppn(&self) -> PPN {
        PPN::new(self.0 >> 10)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & 1 == 1
    }
}

pub struct PageTable(pub &'static mut [PageTableEntry; 512]);

impl PageTable {
    pub fn from(address: PA) -> Self {
        Self(unsafe { &mut *(address.0 as *mut [PageTableEntry; 512]) })
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
