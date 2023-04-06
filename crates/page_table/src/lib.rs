#![no_std]

extern crate alloc;

mod address;
mod page_table;

pub use address::{Page, Address};
use alloc::string::String;
use page_table::PageTableEntryFlags;
pub use address::{VPN, PPN, VA, PA};
use page_table::{PageTableEntry, PageTable};
use core::marker::PhantomData;
use alloc::vec;
use alloc::vec::Vec;

pub const LOW_START_ADDR: usize = 0;
pub const LOW_END_ADDR: usize = 0x4000000000;
pub const HIGH_START_ADDR: usize = 0xffffffc000000000;
pub const HIGH_END_ADDR: usize = 0xfffffffffffff000;

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
    pub fn map(&mut self, vpn: usize, identical: bool, user_accessible: bool) {
        let vpn = VPN::new(vpn);
        let ppn =if identical {
            PPN::new(vpn.number())
        } else {
            let frame = FrameAllocator::alloc();
            self.frames.push(frame);
            frame
        };
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid());
        let flags = if user_accessible {
            PageTableEntryFlags::UXWR
        } else {
            PageTableEntryFlags::XWR
        };
        *pte = PageTableEntry::new(ppn, flags);
    }

    fn translate(&self, vpn: usize) -> PPN {
        let vpn = VPN::new(vpn);
        self.find_pte(vpn).ppn()
    }

    pub fn satp(&self) -> usize {
        1 << 63 | self.root_ppn.number()
    }
}

impl<FrameAllocator: FrameAlloc> SV39PageTable<FrameAllocator> {
    fn translate_addr(&self, start_va: VA, end_va: VA) -> Vec<(PA, PA)> {
        let start_vpn = start_va.align_to_lower().page();
        let end_vpn = end_va.align_to_upper().page();
        (start_vpn.number()..end_vpn.number())
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
                if i == end_vpn.number() - start_vpn.number() - 1 {
                    end_pa = ppn.start_addr().offset(end_va.page_offset());
                } else {
                    end_pa = ppn.end_addr();
                }
                (start_pa, end_pa)
            })
            .collect()
    }

    pub fn translate_buffer(&self, va: usize, len: usize) -> Vec<&'static mut [u8]> {
        let start_va = VA::new(va);
        let end_va = VA::new(va + len);
        self.translate_addr(start_va, end_va)
            .iter()
            .map(|(start_pa, end_pa)| {
                unsafe {
                    core::slice::from_raw_parts_mut(start_pa.number() as *mut u8, end_pa.number() - start_pa.number())
                }
            })
            .collect()
    }

    pub fn translate_one_addr(&self, va: usize) -> usize {
        let va = VA::new(va);
        let vpn = va.align_to_lower().page().number();
        let ppn = self.translate(vpn);
        ppn.start_addr().offset(va.page_offset()).number()
    }

    pub fn translate_type<T>(&self, va: usize) -> &'static mut T {
        let start_va = VA::new(va);
        let len = core::mem::size_of::<T>();
        let end_va = VA::new(va + len);
        let pa_ranges = self.translate_addr(start_va, end_va);
        // TODO: 不知道怎么处理类型没在一个物理页内的情况
        assert!(pa_ranges.len() == 1);
        let start_pa = pa_ranges[0].0;
        unsafe {
            &mut *(start_pa.number() as *mut T)
        }
    }

    pub fn read_str(&self, va: usize, len: usize) -> String {
        let buffer_list = self.translate_buffer(va, len);
        let mut s = String::new();
        for buffer in buffer_list {
            for byte in buffer {
                s.push(byte.clone() as char);
            }
        }
        s
    }

    pub fn write(&self, va: usize, len: usize, data: &[u8]) {
        let buffer_list = self.translate_buffer(va, len);
        let mut i = 0;
        let mut remain_len = data.len();
        for buffer in buffer_list {
            if i >= data.len() {
                break;
            }
            let len = buffer.len().min(remain_len);
            remain_len -= len;
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
