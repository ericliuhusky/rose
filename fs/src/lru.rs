use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use core::cell::RefCell;

struct LinkedHashList<K: Ord + Clone, V> {
    list: VecDeque<Rc<RefCell<V>>>,
    dict: BTreeMap<K, Rc<RefCell<V>>>,
}

impl<K: Ord + Clone, V> LinkedHashList<K, V> {
    fn new() -> Self {
        Self {
            list: VecDeque::new(),
            dict: BTreeMap::new(),
        }
    }

    fn set(&mut self, k: K, v: Rc<RefCell<V>>) {
        if let Some(r) = self.dict.get_mut(&k) {
            *r = v;
        } else {
            self.dict.insert(k, Rc::clone(&v));
            self.list.push_front(Rc::clone(&v));
        }
    }

    fn get(&self, k: &K) -> Option<&Rc<RefCell<V>>> {
        self.dict.get(k)
    }

    fn move_to_fisrt(&mut self, k: &K) {
        if let Some(r) = self.dict.get(k) {
            if let Some((i, _)) = self.list.iter().enumerate().find(|(_, t)| Rc::ptr_eq(t, r)) {
                self.list.remove(i);
            }
            self.list.push_front(Rc::clone(r));
        }
    }

    fn remove_last(&mut self) {
        if let Some(last) = self.list.pop_back() {
            if let Some(k) = self
                .dict
                .iter()
                .find(|(_, t)| Rc::ptr_eq(t, &last))
                .map(|(k, _)| k.clone())
            {
                self.dict.remove(&k);
            }
        }
    }
}

pub struct LRUCache<K: Ord + Clone, V> {
    l: LinkedHashList<K, V>,
    capacity: usize,
}

impl<K: Ord + Clone, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            l: LinkedHashList::new(),
            capacity,
        }
    }

    pub fn set(&mut self, k: K, v: Rc<RefCell<V>>) {
        self.refresh_used(&k);
        self.l.set(k, v);

        if self.l.list.len() > self.capacity {
            self.l.remove_last();
        }
    }

    pub fn get(&mut self, k: &K) -> Option<&Rc<RefCell<V>>> {
        self.refresh_used(k);
        self.l.get(k)
    }

    fn refresh_used(&mut self, k: &K) {
        if self.l.dict.contains_key(k) {
            self.l.move_to_fisrt(k);
        }
    }

    pub fn list(&self) -> &VecDeque<Rc<RefCell<V>>> {
        &self.l.list
    }
}
