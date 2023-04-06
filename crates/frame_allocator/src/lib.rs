#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use page_table::{FrameAlloc, PA, PPN, Address, Page};

struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    fn new(end_pa: usize) -> Self {
        extern "C" {
            // 内核结尾地址
            fn ekernel();
        }
        let current = PA::new(ekernel as usize).align_to_upper().page().number();
        let end = PA::new(end_pa).align_to_lower().page().number();
        Self {
            current,
            end,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> PPN {
        if self.current == self.end {
            panic!()
        }
        if let Some(ppn) = self.recycled.pop() {
            PPN::new(ppn)
        } else {
            let ppn = self.current;
            self.current += 1;
            PPN::new(ppn)
        }
    }

    fn dealloc(&mut self, frame: PPN) {
        unsafe {
            *(frame.start_addr().number() as *mut [u8; 0x1000]) = [0; 0x1000];
        }
        self.recycled.push(frame.number());
    }
}

pub struct FrameAllocator;

impl FrameAlloc for FrameAllocator {
    fn alloc() -> PPN {
        unsafe {
            FRAME_ALLOCATOR.alloc()
        }
    }

    fn dealloc(frame: PPN) {
        unsafe {
            FRAME_ALLOCATOR.dealloc(frame)
        }
    }
}

pub fn alloc() -> usize {
    FrameAllocator::alloc().number()
}

pub fn dealloc(frame: usize) {
    FrameAllocator::dealloc(PPN::new(frame));
}

static mut FRAME_ALLOCATOR: StackFrameAllocator = StackFrameAllocator {
    current: 0,
    end: 0,
    recycled: Vec::new(),
};

pub fn init(end_pa: usize) {
    unsafe {
        FRAME_ALLOCATOR = StackFrameAllocator::new(end_pa);
    }
}
