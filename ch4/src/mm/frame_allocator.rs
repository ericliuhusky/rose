//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use super::{物理页};
use crate::mm::address::{将地址转为页号并向下取整, 将地址转为页号并向上取整};
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
            current: 将地址转为页号并向上取整(ekernel as usize),
            end: 将地址转为页号并向下取整(MEMORY_END)
        }
    }

    fn alloc(&mut self) -> 物理页 {
        if self.current == self.end {
            panic!()
        }
        self.current += 1;
        物理页(self.current - 1)
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
pub fn frame_alloc() -> 物理页 {
    unsafe {
        FRAME_ALLOCATOR.alloc()
    }
}
