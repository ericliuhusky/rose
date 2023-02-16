//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use super::{PhysPageNum};
use crate::mm::address::{floor, ceil};
use crate::config::MEMORY_END;

/// an implementation for frame allocator
struct FrameAllocator {
    current: usize,
    end: usize,
}

impl FrameAllocator {
    fn init() -> Self {
        extern "C" {
            fn ekernel();
        }
        Self {
            current: ceil(ekernel as usize),
            end: floor(MEMORY_END)
        }
    }

    fn alloc(&mut self) -> PhysPageNum {
        if self.current == self.end {
            panic!()
        }
        self.current += 1;
        PhysPageNum(self.current - 1)
    }
}

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {
    current: 0,
    end: 0,
};

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    unsafe {
        FRAME_ALLOCATOR = FrameAllocator::init();
    }
}

/// allocate a frame
pub fn frame_alloc() -> PhysPageNum {
    unsafe {
        FRAME_ALLOCATOR.alloc()
    }
}
