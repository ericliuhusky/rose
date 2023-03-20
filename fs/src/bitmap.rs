use super::{get_block_cache, BlockDevice, BLOCK_SZ};
use alloc::rc::Rc;
/// A bitmap block
type BitmapBlock = [u64; 64];
/// Number of bits in a block
const BLOCK_BITS: usize = BLOCK_SZ * 8;
/// A bitmap
pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

/// Decompose bits into (block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}

impl Bitmap {
    /// A new bitmap from start block id and number of blocks
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }
    /// Allocate a new block from a block device
    pub fn alloc(&self, block_device: Rc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id as usize,
                Rc::clone(&block_device),
            )
            .borrow_mut()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some(i) = (0..bitmap_block.len()).find(|i| bitmap_block[*i] != u64::MAX) {
                    let j = bitmap_block[i].trailing_ones() as usize;
                    bitmap_block[i] |= 1 << j;
                    Some(block_id * BLOCK_BITS + i * 64 + j)
                } else {
                    None
                }
            });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }
    /// Deallocate a block
    pub fn dealloc(&self, block_device: &Rc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, Rc::clone(block_device))
            .borrow_mut()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                bitmap_block[bits64_pos] &= !(1 << inner_pos);
            });
    }
    /// Get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
