use alloc::vec::Vec;

pub struct IDAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl IDAllocator {
    pub const fn new() -> Self {
        Self { 
            current: 0, 
            recycled: Vec::new() 
        }
    }

    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            let id = self.current;
            self.current += 1;
            id
        }
    }

    fn dealloc(&mut self, id: usize) {
        self.recycled.push(id);
    }
}

static mut PID_ALLOCATOR: IDAllocator = IDAllocator::new();

pub fn pid_alloc() -> Pid {
    Pid(unsafe { PID_ALLOCATOR.alloc() })
}

pub struct Pid(pub usize);

impl Drop for Pid {
    fn drop(&mut self) {
        unsafe {
            PID_ALLOCATOR.dealloc(self.0);
        }
    }
}
