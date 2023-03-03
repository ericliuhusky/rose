#![no_std]

extern crate alloc;

mod address;
mod page_table;

pub use page_table::PageTableEntryFlags;
pub use address::{VPN, PPN, VA, PA};
use page_table::{PageTableEntry, PageTable};
use core::marker::PhantomData;
use alloc::vec;
use alloc::vec::Vec;

pub trait FrameAlloc {
    fn alloc() -> PPN;
    fn dealloc(frame: PPN);
}

pub struct SV39PageTable<FrameAllocator: FrameAlloc> {
    pub root_ppn: PPN,
    frames: Vec<PPN>,
    _phantom: PhantomData<FrameAllocator>
}

impl<FrameAllocator: FrameAlloc> SV39PageTable<FrameAllocator> {
    pub fn new() -> Self {
        let frame = FrameAllocator::alloc(); 
        Self { 
            root_ppn: frame, 
            frames: vec![frame],
            _phantom: PhantomData
        }
    }
}

impl<FrameAllocator: FrameAlloc> SV39PageTable<FrameAllocator> {
    // ppn -> pa -> page_table -> pte -> ppn
    fn find_pte_create(&mut self, vpn: VPN) -> &mut PageTableEntry {
        let indexs = vpn.indexs();
        let mut ppn = self.root_ppn;
        for i in 0..2 {
            let pa = ppn.start_addr();
            let mut pt = PageTable::from(pa);
            let pte = &mut pt[indexs[i]];
            if pte.is_valid() {
                ppn = pte.ppn();
            } else {
                let frame = FrameAllocator::alloc();
                ppn = frame;
                self.frames.push(frame);
                pt[indexs[i]] = PageTableEntry::new(ppn, PageTableEntryFlags::POINT_TO_NEXT);
            }
        }
        let pa = ppn.start_addr();
        let pt = PageTable::from(pa);
        let pte = &mut pt.0[indexs[2]];
        pte
    }

    fn find_pte(&self, vpn: VPN) -> &mut PageTableEntry {
        let indexs = vpn.indexs();
        let mut ppn = self.root_ppn;
        for i in 0..2 {
            let pa = ppn.start_addr();
            let mut pt = PageTable::from(pa);
            let pte = &mut pt[indexs[i]];
            if pte.is_valid() {
                ppn = pte.ppn();
            } else {
                panic!()
            }
        }
        let pa = ppn.start_addr();
        let pt = PageTable::from(pa);
        let pte = &mut pt.0[indexs[2]];
        pte
    }
}

impl<FrameAllocator: FrameAlloc> SV39PageTable<FrameAllocator> {
    pub fn map(&mut self, vpn: VPN, identical: bool, flags: PageTableEntryFlags) {
        let ppn;
        if identical {
            ppn = PPN::new(vpn.0);
        } else {
            let frame = FrameAllocator::alloc();
            self.frames.push(frame);
            ppn = frame;
        }
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid());
        *pte = PageTableEntry::new(ppn, flags);
    }

    pub fn translate(&self, vpn: VPN) -> PPN {
        self.find_pte(vpn).ppn()
    }
}

impl<FrameAllocator: FrameAlloc> SV39PageTable<FrameAllocator> {
    pub fn translate_addr(&self, start_va: VA, end_va: VA) -> Vec<(PA, PA)> {
        let start_vpn = start_va.align_to_lower().page_number();
        let end_vpn = end_va.align_to_upper().page_number();
        (start_vpn.0..end_vpn.0)
            .map(|vpn| VPN::new(vpn))
            .map(|vpn| {
                self.translate(vpn)
            })
            .enumerate()
            .map(|(i, ppn)| {
                let start_pa;
                if i == 0 {
                    start_pa = ppn.start_addr().offset(start_va.page_offset());
                } else {
                    start_pa = ppn.start_addr();
                }
                let end_pa;
                if i == end_vpn.0 - start_vpn.0 - 1 {
                    end_pa = ppn.start_addr().offset(end_va.page_offset());
                } else {
                    end_pa = ppn.end_addr();
                }
                (start_pa, end_pa)
            })
            .collect()
    }

    fn translate_buffer(&self, start_va: VA, end_va: VA) -> Vec<&'static mut [u8]> {
        self.translate_addr(start_va, end_va)
            .iter()
            .map(|(start_pa, end_pa)| {
                unsafe {
                    core::slice::from_raw_parts_mut(start_pa.0 as *mut u8, end_pa.0 - start_pa.0)
                }
            })
            .collect()
    }

    pub fn read(&self, start_va: VA, end_va: VA) -> Vec<u8> {
        let buffer_list = self.translate_buffer(start_va, end_va);
        // let mut i = 0;
        // for buffer in buffer_list {
        //     for byte in buffer {
        //         data[i] = *byte;
        //         i += 1;
        //     }
        // }
        let mut v = Vec::new();
        for buffer in buffer_list {
            for byte in buffer {
                v.push(byte.clone());
            }
        }
        v
    }

    pub fn write(&self, start_va: VA, end_va: VA, data: &[u8]) {
        let buffer_list = self.translate_buffer(start_va, end_va);
        let mut i = 0;
        for buffer in buffer_list {
            if i >= data.len() {
                break;
            }
            let len = buffer.len().min(data.len());
            let src = &data[i..i+len];
            i += len;
            for j in 0..len {
                buffer[j] = src[j];
            }
        }
    }
}

impl<FrameAllocator: FrameAlloc> Drop for SV39PageTable<FrameAllocator> {
    fn drop(&mut self) {
        for frame in &self.frames {
            FrameAllocator::dealloc(*frame);
        }
    }
}
