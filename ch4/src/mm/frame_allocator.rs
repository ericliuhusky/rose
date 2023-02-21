//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use crate::mm::address::{将地址转为页号并向下取整, 将地址转为页号并向上取整, 物理页};
use crate::config::MEMORY_END;

/// an implementation for frame allocator
pub struct FrameAllocator {
    current: usize,
    end: usize,
}

impl FrameAllocator {
    /// initiate the frame allocator using `ekernel` and `MEMORY_END`
    pub fn init_frame_allocator() {
        extern "C" {
            fn ekernel();
        }
        unsafe {
            FRAME_ALLOCATOR = Self {
                current: 将地址转为页号并向上取整(ekernel as usize) - 1,
                end: 将地址转为页号并向下取整(MEMORY_END)
            };
        }
    }

    /// allocate a frame
    pub fn frame_alloc() -> 物理页 {
        unsafe {
            if FRAME_ALLOCATOR.current == FRAME_ALLOCATOR.end {
                panic!()
            }
            FRAME_ALLOCATOR.current += 1;
            物理页(FRAME_ALLOCATOR.current)
        }
    }
}

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {
    current: 0,
    end: 0,
};
