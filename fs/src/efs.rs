use super::{
    block_cache_sync_all, get_block_cache, Bitmap, BlockDevice, DiskInode, DiskInodeType, Inode,
    SuperBlock,
};
use crate::BLOCK_SZ;
use alloc::rc::Rc;
use core::cell::RefCell;
use super::*;
///An file system on block
pub struct FileSystem {
    ///Real device
    pub block_device: Rc<dyn BlockDevice>,
    ///Inode bitmap
    pub inode_bitmap: Bitmap,
    ///Data bitmap
    pub data_bitmap: Bitmap,
    inode_area_start_block: u32,
    data_area_start_block: u32,
}

type DataBlock = [u8; BLOCK_SZ];
/// An fs over a block device
impl FileSystem {
    /// A data block of block size
    pub fn create(
        block_device: Rc<dyn BlockDevice>,
    ) -> Rc<RefCell<Self>> {
        let inode_bitmap = Bitmap::new(1, INODE_BITMAP_BLOCK_NUM as usize);
        let data_bitmap = Bitmap::new((1 + INODE_BITMAP_BLOCK_NUM + INODE_AREA_BLOCK_NUM) as usize, DATA_BITMAP_BLOCK_NUM as usize);
        let mut efs = Self {
            block_device: Rc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_area_start_block: 1 + INODE_BITMAP_BLOCK_NUM,
            data_area_start_block: 1 + INODE_BITMAP_BLOCK_NUM + INODE_AREA_BLOCK_NUM + DATA_BITMAP_BLOCK_NUM,
        };
        // clear all blocks
        for i in 0..TOTAL_BLOCK_NUM {
            get_block_cache(i as usize, Rc::clone(&block_device))
                .borrow_mut()
                .modify(0, |data_block: &mut DataBlock| {
                    for byte in data_block.iter_mut() {
                        *byte = 0;
                    }
                });
        }
        // initialize SuperBlock
        get_block_cache(0, Rc::clone(&block_device)).borrow_mut()
        .set(0,
             SuperBlock::new(
            INODE_BITMAP_BLOCK_NUM, 
            INODE_AREA_BLOCK_NUM, 
            DATA_BITMAP_BLOCK_NUM,
            DATA_AREA_BLOCK_NUM));

        // write back immediately
        // create a inode for root node "/"
        assert_eq!(efs.alloc_inode(), 0);
        let (root_inode_block_id, root_inode_offset) = efs.get_disk_inode_pos(0);
        get_block_cache(root_inode_block_id as usize, Rc::clone(&block_device))
            .borrow_mut()
            .modify(root_inode_offset, |disk_inode: &mut DiskInode| {
                disk_inode.initialize(DiskInodeType::Directory);
            });
        block_cache_sync_all();
        Rc::new(RefCell::new(efs))
    }
    /// Open a block device as a filesystem
    pub fn open(block_device: Rc<dyn BlockDevice>) -> Rc<RefCell<Self>> {
        // read SuperBlock
        get_block_cache(0, Rc::clone(&block_device))
            .borrow()
            .read(0, |super_block: &SuperBlock| {
                let inode_total_blocks =
                    super_block.inode_bitmap_block_num + super_block.inode_area_block_num;
                let efs = Self {
                    block_device,
                    inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_block_num as usize),
                    data_bitmap: Bitmap::new(
                        (1 + inode_total_blocks) as usize,
                        super_block.data_bitmap_block_num as usize,
                    ),
                    inode_area_start_block: 1 + super_block.inode_bitmap_block_num,
                    data_area_start_block: 1 + inode_total_blocks + super_block.data_bitmap_block_num,
                };
                Rc::new(RefCell::new(efs))
            })
    }
    /// Get the root inode of the filesystem
    pub fn root_inode(efs: &Rc<RefCell<Self>>) -> Inode {
        let block_device = Rc::clone(&efs.borrow().block_device);
        // acquire efs lock temporarily
        let (block_id, block_offset) = efs.borrow().get_disk_inode_pos(0);
        // release efs lock
        Inode::new(block_id, block_offset, Rc::clone(efs), block_device)
    }
    /// Get inode by id
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inodes_per_block = (BLOCK_SZ / inode_size) as u32;
        let block_id = self.inode_area_start_block + inode_id / inodes_per_block;
        (
            block_id,
            (inode_id % inodes_per_block) as usize * inode_size,
        )
    }
    /// Get data block by id
    pub fn get_data_block_id(&self, data_block_id: u32) -> u32 {
        self.data_area_start_block + data_block_id
    }
    /// Allocate a new inode
    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(self.block_device.clone()).unwrap() as u32
    }

    /// Allocate a data block
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(self.block_device.clone()).unwrap() as u32 + self.data_area_start_block
    }
    /// Deallocate a data block
    pub fn dealloc_data(&mut self, block_id: u32) {
        get_block_cache(block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|p| {
                    *p = 0;
                })
            });
        self.data_bitmap.dealloc(
            &self.block_device,
            (block_id - self.data_area_start_block) as usize,
        )
    }
}
