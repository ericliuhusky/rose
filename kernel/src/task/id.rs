use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct IDAllocDict<V> {
    dict: BTreeMap<usize, V>,
    id_allocator: IDAllocator,
}

impl<V> IDAllocDict<V> {
    pub fn new() -> Self {
        Self { 
            dict: BTreeMap::new(),
            id_allocator: IDAllocator::new() 
        }
    }

    pub fn insert(&mut self, value: V) -> usize {
        let id = self.id_allocator.alloc();
        self.dict.insert(id, value);
        id
    }

    pub fn get(&self, id: usize) -> Option<&V> {
        self.dict.get(&id)
    }

    pub fn remove(&mut self, id: usize) {
        self.dict.remove(&id);
    }
}
