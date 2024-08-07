use alloc_ext::collections::LRUCache;
use super::{BlockDevice, BLOCK_SZ};
use alloc::rc::Rc;
use core::cell::RefCell;

/// Cached block inside memory
pub struct BlockCache {
    /// cached block data
    cache: [u8; BLOCK_SZ],
    /// underlying block id
    block_id: usize,
    /// underlying block device
    block_device: Rc<dyn BlockDevice>,
    /// whether the block is dirty
    modified: bool,
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(block_id: usize, block_device: Rc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SZ];
        block_device.read_block(block_id, &mut cache);
        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }

    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        let addr = &self.cache[offset] as *const u8 as usize;
        f(unsafe { &*(addr as *const T) })
    }

    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        self.modified = true;
        let addr = self.cache.as_mut_ptr();
        f(unsafe { &mut *(addr as *mut T) })
    }

    pub fn get<V>(&self, offset: usize) -> &V {
        let addr = &self.cache[offset] as *const u8 as usize;
        unsafe { &*(addr as *const V) }
    }

    pub fn set<V>(&mut self, offset: usize, value: V) {
        self.modified = true;
        let addr = self.cache.as_mut_ptr();
        unsafe {
            *(addr as *mut V) = value;
        }
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}
/// Use a block cache of 16 blocks
const BLOCK_CACHE_SIZE: usize = 16;

pub struct BlockCacheManager {
    lru: LRUCache<usize, Rc<RefCell<BlockCache>>>,
}

impl BlockCacheManager {
    fn new() -> Self {
        Self { lru: LRUCache::new(BLOCK_CACHE_SIZE) }
    }

    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Rc<dyn BlockDevice>,
    ) -> Rc<RefCell<BlockCache>> {
        if let Some(v) = self.lru.get(&block_id) {
            Rc::clone(v)
        } else {
            let block_cache = Rc::new(RefCell::new(BlockCache::new(
                block_id,
                Rc::clone(&block_device),
            )));
            self.lru.set(block_id, Rc::clone(&block_cache));
            block_cache
        }
    }
}

lazy_static::lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: RefCell<BlockCacheManager> = RefCell::new(BlockCacheManager::new());
}

/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(
    block_id: usize,
    block_device: Rc<dyn BlockDevice>,
) -> Rc<RefCell<BlockCache>> {
    BLOCK_CACHE_MANAGER.borrow_mut().get_block_cache(block_id, block_device)
}
/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    for cache in BLOCK_CACHE_MANAGER.borrow().lru.values().iter() {
        cache.borrow_mut().sync();
    }
}
