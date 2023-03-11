use super::{BlockDevice, BLOCK_SZ};
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
/// Cached block inside memory
pub struct BlockCache {
    /// cached block data
    cache: [u8; BLOCK_SZ],
    /// underlying block id
    block_id: usize,
    /// underlying block device
    block_device: Arc<dyn BlockDevice>,
    /// whether the block is dirty
    modified: bool,
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
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
        let addr = &self.cache[offset] as *const u8 as usize;
        f(unsafe { &mut *(addr as *mut T) })
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
    queue: Vec<(usize, Rc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Arc<dyn BlockDevice>,
    ) -> Rc<Mutex<BlockCache>> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Rc::clone(&pair.1)
        } else {
            // substitute
            if self.queue.len() == BLOCK_CACHE_SIZE {
                // from front to tail
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Rc::strong_count(&pair.1) == 1)
                {
                    self.queue.remove(idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // load block into mem and push back
            let block_cache = Rc::new(Mutex::new(BlockCache::new(
                block_id,
                Arc::clone(&block_device),
            )));
            self.queue.push((block_id, Rc::clone(&block_cache)));
            block_cache
        }
    }
}

static mut BLOCK_CACHE_MANAGER: BlockCacheManager = BlockCacheManager {
    queue: Vec::new(),
};
/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Rc<Mutex<BlockCache>> {
    unsafe {
        BLOCK_CACHE_MANAGER.get_block_cache(block_id, block_device)
    }
}
/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    unsafe {
        for (_, cache) in BLOCK_CACHE_MANAGER.queue.iter() {
            cache.lock().sync();
        }
    }
}
