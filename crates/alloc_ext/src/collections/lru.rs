use alloc::{
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};

struct OrderedDictionary<K: Ord + Clone, V> {
    keys: VecDeque<K>,
    dict: BTreeMap<K, V>,
}

impl<K: Ord + Clone, V> OrderedDictionary<K, V> {
    fn new() -> Self {
        Self {
            keys: VecDeque::new(),
            dict: BTreeMap::new(),
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.dict.get(key)
    }

    fn set(&mut self, key: K, value: V) {
        if !self.dict.contains_key(&key) {
            self.keys.push_back(key.clone());
        }
        self.dict.insert(key, value);
    }

    fn remove(&mut self, key: &K) {
        if self.dict.contains_key(key) {
            let (index, _) = self
                .keys
                .iter()
                .enumerate()
                .find(|(_, _k)| **_k == *key)
                .unwrap();
            self.keys.remove(index);
        }
        self.dict.remove(key);
    }

    fn values(&self) -> Vec<&V> {
        let mut v = Vec::new();
        for key in &self.keys {
            v.push(self.dict.get(key).unwrap());
        }
        v
    }
}

pub struct LRUCache<K: Ord + Clone, V> {
    dict: OrderedDictionary<K, V>,
    capacity: usize,
}

impl<K: Ord + Clone, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            dict: OrderedDictionary::new(),
            capacity,
        }
    }

    fn refresh(&mut self, k: &K) {
        if let Some((index, _)) = self.dict.keys.iter().enumerate().find(|(_, _k)| **_k == *k) {
            self.dict.keys.remove(index);
            self.dict.keys.push_back(k.clone());
        }
    }

    fn remove_least_used(&mut self) {
        if let Some(key) = self.dict.keys.front().map(|x| x.clone()) {
            self.dict.remove(&key);
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.refresh(key);
        self.dict.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        if self.dict.get(&key).is_some() {
            self.refresh(&key);
        } else {
            if self.dict.keys.len() == self.capacity {
                self.remove_least_used();
            }
        }
        self.dict.set(key, value);
    }

    pub fn values(&self) -> Vec<&V> {
        self.dict.values()
    }
}
