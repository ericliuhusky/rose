use super::{get_block_cache, BlockDevice, BLOCK_SZ};
use alloc::rc::Rc;
use core::mem::size_of;

const BLOCK_BITS: usize = BLOCK_SZ * 8;

struct Dimension(u128);

impl Dimension {
    fn is_full(&self) -> bool {
        self.0 == u128::MAX
    }

    fn allocated(&self) -> (usize, Self) {
        let first_zero_bit_index = self.0.trailing_ones() as usize;
        (
            first_zero_bit_index,
            Self(self.0 | 1 << first_zero_bit_index),
        )
    }

    fn deallocated(&self, bit_index: usize) -> Self {
        Self(self.0 & !(1 << bit_index))
    }

    fn bit_size() -> usize {
        size_of::<Self>() * 8
    }
}

struct BitmapIndex(usize);

impl BitmapIndex {
    fn new(block_index: usize, dimension_index: usize, bit_index: usize) -> Self {
        Self(block_index * BLOCK_BITS + dimension_index * Dimension::bit_size() + bit_index)
    }

    fn decompose(&self) -> (usize, usize, usize) {
        let block_index = self.0 / BLOCK_BITS;
        let remain_index = self.0 % BLOCK_BITS;
        let dimension_index = remain_index / Dimension::bit_size();
        let bit_index = remain_index % Dimension::bit_size();
        (block_index, dimension_index, bit_index)
    }
}

pub struct Bitmap {
    start_block_index: usize,
    block_num: usize,
}

impl Bitmap {
    pub fn new(start_block_index: usize, block_num: usize) -> Self {
        Self {
            start_block_index,
            block_num,
        }
    }

    pub fn alloc(&self, block_device: Rc<dyn BlockDevice>) -> Option<usize> {
        for inner_block_index in 0..self.block_num {
            let bit_map_index = get_block_cache(
                inner_block_index + self.start_block_index as usize,
                Rc::clone(&block_device),
            )
            .borrow_mut()
            .modify(0, |bitmap_block: &mut [Dimension; 32]| {
                if let Some((dimension_index, dimension)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, dimension)| !dimension.is_full())
                {
                    let (bit_index, dimension) = dimension.allocated();
                    bitmap_block[dimension_index] = dimension;
                    Some(BitmapIndex::new(inner_block_index, dimension_index, bit_index).0)
                } else {
                    None
                }
            });
            if bit_map_index.is_some() {
                return bit_map_index;
            }
        }
        None
    }

    pub fn dealloc(&self, block_device: &Rc<dyn BlockDevice>, bit: usize) {
        let (inner_block_index, dimension_index, bit_index) = BitmapIndex(bit).decompose();
        get_block_cache(
            inner_block_index + self.start_block_index,
            Rc::clone(block_device),
        )
        .borrow_mut()
        .modify(0, |bitmap_block: &mut [Dimension; 32]| {
            let dimension = bitmap_block[dimension_index].deallocated(bit_index);
            bitmap_block[dimension_index] = dimension;
        });
    }
}
