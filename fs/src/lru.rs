use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::collections::{BTreeMap, VecDeque};

struct Node {
    k: usize,
    v: usize,
}

struct LinkedHashList {
    list: VecDeque<Rc<RefCell<Node>>>,
    dict: BTreeMap<usize, Rc<RefCell<Node>>>,
}

impl LinkedHashList {
    fn new() -> Self {
        Self {
            list: VecDeque::new(),
            dict: BTreeMap::new(),
        }
    }

    fn set(&mut self, k: usize, v: usize) {
        if let Some(n) = self.dict.get(&k) {
            n.borrow_mut().v = v;
        } else {
            let n = Rc::new(RefCell::new(Node { k, v }));
            self.dict.insert(k, Rc::clone(&n));
            self.list.push_front(Rc::clone(&n));
        }
    }

    fn get(&self, k: usize) -> Option<usize> {
        self.dict.get(&k).map(|n| n.borrow().v)
    }

    fn move_to_fisrt(&mut self, k: usize) {
        if let Some(n) = self.dict.get(&k) {
            if let Some((i, _)) = self
                .list
                .iter()
                .enumerate()
                .find(|(_, t)| t.borrow().v == n.borrow().v)
            {
                self.list.remove(i);
            }
            self.list.push_front(Rc::clone(n));
        }
    }

    fn remove_last(&mut self) {
        if let Some(last) = self.list.pop_back() {
            self.dict.remove(&last.borrow().k);
        }
    }
}

struct LRUCache {
    l: LinkedHashList,
    capacity: usize,
}

impl LRUCache {
    fn new(capacity: usize) -> Self {
        Self {
            l: LinkedHashList::new(),
            capacity,
        }
    }

    fn set(&mut self, k: usize, v: usize) {
        self.refresh_used(k);
        self.l.set(k, v);

        if self.l.list.len() > self.capacity {
            self.l.remove_last();
        }
    }

    fn get(&mut self, k: usize) -> Option<usize> {
        self.refresh_used(k);
        self.l.get(k)
    }

    fn refresh_used(&mut self, k: usize) {
        if self.l.dict.contains_key(&k) {
            self.l.move_to_fisrt(k);
        }
    }
}
