use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Clone)]
struct IDAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl IDAllocator {
    const fn new() -> Self {
        Self {
            current: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> usize {
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

#[derive(Clone)]
pub struct IDAllocDict<V> {
    dict: BTreeMap<usize, V>,
    id_allocator: IDAllocator,
}

impl<V> IDAllocDict<V> {
    pub const fn new() -> Self {
        Self {
            dict: BTreeMap::new(),
            id_allocator: IDAllocator::new(),
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
        self.id_allocator.dealloc(id);
    }
}
