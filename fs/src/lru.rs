use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use core::cell::RefCell;

struct LinkedHashList<K: Ord + Clone, V> {
    keys: VecDeque<K>,
    dict: BTreeMap<K, Rc<RefCell<V>>>,
}

impl<K: Ord + Clone, V> LinkedHashList<K, V> {
    fn new() -> Self {
        Self {
            keys: VecDeque::new(),
            dict: BTreeMap::new(),
        }
    }

    fn get(&self, k: &K) -> Option<&Rc<RefCell<V>>> {
        self.dict.get(k)
    }

    fn set(&mut self, k: K, v: Rc<RefCell<V>>) {
        if !self.dict.contains_key(&k) {
            self.keys.push_back(k.clone());
        }
        self.dict.insert(k, v.clone());
    }

    fn remove(&mut self, k: &K) {
        if self.dict.contains_key(k) {
            let (i, _) = self.keys.iter().enumerate().find(|(_, _k)| **_k == k.clone()).unwrap();
            self.keys.remove(i);
        }
        self.dict.remove(k);
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

    fn refresh(&mut self, k: K) {
        if let Some((i, _)) = self.l.keys.iter().enumerate().find(|(_, _k)| **_k == k) {
            self.l.keys.remove(i);
            self.l.keys.push_back(k);
        }
    }

    fn remove_least_used(&mut self) {
        if let Some(k) = self.l.keys.front().map(|x| x.clone()) {
            self.l.remove(&k);
        }
    }

    pub fn set(&mut self, k: K, v: Rc<RefCell<V>>) {
        if self.l.get(&k).is_some() {
            self.refresh(k.clone());
        } else {
            if self.l.keys.len() == self.capacity {
                self.remove_least_used();
            }
        }
        self.l.set(k, v);
    }

    pub fn get(&mut self, k: &K) -> Option<&Rc<RefCell<V>>> {
        self.refresh(k.clone());
        self.l.get(k)
    }

    pub fn list(&self) -> VecDeque<Rc<RefCell<V>>> {
        self.l.keys.iter().map(|k| self.l.dict[k].clone()).collect()
    }
}
