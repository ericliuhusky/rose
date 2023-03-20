use super::{get_block_cache, BlockDevice, BLOCK_SZ};
use alloc::rc::Rc;
use alloc::vec::Vec;

/// The max number of direct inodes
const INODE_DIRECT_COUNT: usize = 63;
/// The max number of indirect inodes
const INODE_INDIRECT_COUNT: usize = BLOCK_SZ / 4;
/// A indirect block
type IndirectBlock = [u32; BLOCK_SZ / 4];
/// A data block
type DataBlock = [u8; BLOCK_SZ];

/// A disk inode
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; 63],
    pub indirect: [u32; 64],
}

impl DiskInode {
    pub fn new() -> Self {
        Self { 
            size: 0,
            direct: [0; INODE_DIRECT_COUNT],
            indirect: [0; 64],
        }
    }
    /// Return block number correspond to size.
    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }
    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }
    /// Return number of blocks needed include indirect.
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        if data_blocks > INODE_DIRECT_COUNT {
            total += (data_blocks - INODE_DIRECT_COUNT + INODE_INDIRECT_COUNT - 1) / INODE_INDIRECT_COUNT;
        }
        total as u32
    }
    /// Get the number of data blocks that have to be allocated given the new size of data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }
    /// Get id of block given inner id
    pub fn get_block_id(&self, inner_id: u32, block_device: &Rc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else {
            let indirect_id = inner_id - INODE_DIRECT_COUNT;
            let i = indirect_id / INODE_INDIRECT_COUNT;
            let j = indirect_id % INODE_INDIRECT_COUNT;
            let cache = get_block_cache(self.indirect[i] as usize, Rc::clone(block_device));
            let cache = cache.borrow();
            let indirect_block = cache
                .get::<IndirectBlock>(0);
            indirect_block[j]
        }
    }
    /// Inncrease the size of current disk inode
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Rc<dyn BlockDevice>,
    ) {
        let mut current_blocks = self.data_blocks();
        self.size = new_size;
        let total_blocks = self.data_blocks();
        let mut new_blocks = new_blocks.into_iter();
        for i in current_blocks..total_blocks.min(INODE_DIRECT_COUNT as u32) {
            self.direct[i as usize] = new_blocks.next().unwrap();
            current_blocks += 1;
        }
        if total_blocks > INODE_DIRECT_COUNT as u32 {
            let i0 = (current_blocks as usize - INODE_DIRECT_COUNT) / INODE_INDIRECT_COUNT;
            let i1 = (total_blocks as usize - INODE_DIRECT_COUNT) / INODE_INDIRECT_COUNT;
            let j0 = (current_blocks as usize - INODE_DIRECT_COUNT) % INODE_INDIRECT_COUNT;
            let j1 = (total_blocks as usize - INODE_DIRECT_COUNT) % INODE_INDIRECT_COUNT;
            for i in i0..=i1 {
                let js = if i == i0 { j0 } else { 0 };
                let je = if i == i1 { j1 } else { INODE_INDIRECT_COUNT };
                if js == 0 {
                    self.indirect[i] = new_blocks.next().unwrap();
                }
                get_block_cache(self.indirect[i] as usize, Rc::clone(block_device))
                    .borrow_mut()
                    .modify(0, |indirect_block: &mut IndirectBlock| {
                        for j in js..je {
                            indirect_block[j] = new_blocks.next().unwrap();
                        }
                    });
            }
        }
    }

    /// Clear size to zero and return blocks that should be deallocated.
    /// We will clear the block contents to zero later.
    pub fn clear_size(&mut self, block_device: &Rc<dyn BlockDevice>) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        let data_blocks = self.data_blocks() as usize;
        self.size = 0;
        let mut current_blocks = 0usize;

        while current_blocks < data_blocks.min(INODE_DIRECT_COUNT) {
            v.push(self.direct[current_blocks]);
            self.direct[current_blocks] = 0;
            current_blocks += 1;
        }

        if data_blocks > INODE_DIRECT_COUNT {
            let i1 = (data_blocks as usize - INODE_DIRECT_COUNT) / INODE_INDIRECT_COUNT;
            let j1 = (data_blocks as usize - INODE_DIRECT_COUNT) % INODE_INDIRECT_COUNT;
            for i in 0..=i1 {
                let je = if i == i1 { j1 } else { INODE_INDIRECT_COUNT };
                v.push(self.indirect[i]);
                get_block_cache(self.indirect[i] as usize, Rc::clone(block_device))
                    .borrow_mut()
                    .modify(0, |indirect_block: &mut IndirectBlock| {
                        for j in 0..je {
                            v.push(indirect_block[j]);
                        }
                    });
                self.indirect[i] = 0;
            }
        }
        v
    }
    /// Read data from current disk inode
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Rc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if start >= end {
            return 0;
        }
        let mut start_block = start / BLOCK_SZ;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize,
                Rc::clone(block_device),
            )
            .borrow()
            .read(0, |data_block: &DataBlock| {
                let src = &data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_read_size];
                dst.copy_from_slice(src);
            });
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }
    /// Write data into current disk inode
    /// size must be adjusted properly beforehand
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Rc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SZ;
        let mut write_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let block_write_size = end_current_block - start;
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize,
                Rc::clone(block_device),
            )
            .borrow_mut()
            .modify(0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + block_write_size];
                let dst = &mut data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_write_size];
                dst.copy_from_slice(src);
            });
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }
}
