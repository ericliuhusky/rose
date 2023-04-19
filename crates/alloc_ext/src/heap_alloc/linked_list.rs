use core::ptr;

#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn push(&mut self, item: usize) {
        let node = item as *mut usize;
        unsafe {
            *node = self.head as usize;
        }
        self.head = node;
    }

    pub fn pop(&mut self) -> usize {
        let node = self.head;
        self.head = unsafe { *node as *mut usize };
        let item = node as usize;
        item
    }

    pub fn remove(&mut self, item: usize) -> bool {
        let dummy = (&self.head as *const *mut usize) as *mut usize;
        let mut pre = dummy;
        let mut cur = self.head;
        while !cur.is_null() {
            if cur as usize == item {
                unsafe {
                    *pre = *cur;
                }
                return true;
            }
            pre = cur;
            cur = unsafe { *cur as *mut usize };
        }
        false
    }
}
