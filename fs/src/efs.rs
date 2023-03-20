use super::{
    block_cache_sync_all, get_block_cache, Bitmap, BlockDevice,
    SuperBlock,
};
use inode::{DiskInode, DiskInodeType};
use crate::BLOCK_SZ;
use alloc::rc::Rc;
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
        inode_bitmap_block_num: u32,
        inode_area_block_num: u32,
        data_bitmap_block_num: u32,
        data_area_block_num: u32,
    ) -> Self {
        let inode_bitmap = Bitmap::new(1, inode_bitmap_block_num as usize);
        let data_bitmap = Bitmap::new((1 + inode_bitmap_block_num + inode_area_block_num) as usize, data_bitmap_block_num as usize);
        let mut efs = Self {
            block_device: Rc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_area_start_block: 1 + inode_bitmap_block_num,
            data_area_start_block: 1 + inode_bitmap_block_num + inode_area_block_num + data_bitmap_block_num,
        };
        let total_block_num = inode_bitmap_block_num + inode_area_block_num + data_bitmap_block_num + data_area_block_num;
        // clear all blocks
        for i in 0..total_block_num {
            get_block_cache(i as usize, Rc::clone(&block_device))
                .borrow_mut()
                .modify(0, |data_block: &mut DataBlock| {
                    for byte in data_block.iter_mut() {
                        *byte = 0;
                    }
                });
        }
        get_block_cache(0, Rc::clone(&block_device)).borrow_mut()
        .set(0,
             SuperBlock::new(
            inode_bitmap_block_num, 
            inode_area_block_num, 
            data_bitmap_block_num,
            data_area_block_num));

        // write back immediately
        // create a inode for root node "/"
        assert_eq!(efs.alloc_inode(), 0);
        let root_inode_block_id = efs.get_inode_block_id(0);
        get_block_cache(root_inode_block_id as usize, Rc::clone(&block_device))
            .borrow_mut()
            .set(0, DiskInode::new(DiskInodeType::Directory));
        
        block_cache_sync_all();
        efs
    }
    /// Open a block device as a filesystem
    pub fn open(block_device: Rc<dyn BlockDevice>) -> Self {
        // read SuperBlock
        let cache = get_block_cache(0, Rc::clone(&block_device));
        let cache = cache.borrow();
        let super_block = cache.get::<SuperBlock>(0);
        
        let inode_total_blocks = super_block.inode_bitmap_block_num + super_block.inode_area_block_num;
        let fs = Self {
            block_device,
            inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_block_num as usize),
            data_bitmap: Bitmap::new(
                (1 + inode_total_blocks) as usize,
                super_block.data_bitmap_block_num as usize,
            ),
            inode_area_start_block: 1 + super_block.inode_bitmap_block_num,
            data_area_start_block: 1 + inode_total_blocks + super_block.data_bitmap_block_num,
        };
        fs
    }

    pub fn get_inode_block_id(&self, inode_id: u32) -> u32 {
        self.inode_area_start_block + inode_id
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


use alloc::string::String;
use alloc::vec::Vec;

impl FileSystem {
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<usize> {
        let root_inode_block_id = self.get_inode_block_id(0) as usize;
        get_block_cache(root_inode_block_id, Rc::clone(&self.block_device))
            .borrow()
            .read(0, |disk_inode| {
                self.find_inode_id(name, disk_inode).map(|inode_id| {
                    let block_id = self.get_inode_block_id(inode_id);
                    block_id as usize
                })
            })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &mut self,
        new_size: u32,
        disk_inode: &mut DiskInode
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(self.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// Create inode under current inode by name
    pub fn create_inode(&mut self, name: &str) -> Option<usize> {
        let root_inode_block_id = self.get_inode_block_id(0) as usize;
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if get_block_cache(root_inode_block_id, Rc::clone(&self.block_device)).borrow().read(0, op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = self.alloc_inode();
        let new_inode_block_id = self.get_inode_block_id(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
            .set(0, DiskInode::new(DiskInodeType::File));
        
        get_block_cache(root_inode_block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |root_inode: &mut DiskInode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let block_id = self.get_inode_block_id(new_inode_id);
        block_cache_sync_all();
        Some(block_id as usize)
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let root_inode_block_id = self.get_inode_block_id(0) as usize;
        get_block_cache(root_inode_block_id, Rc::clone(&self.block_device))
            .borrow().read(0, |disk_inode: &DiskInode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, inode_block_id: usize, offset: usize, buf: &mut [u8]) -> usize {
        get_block_cache(inode_block_id, Rc::clone(&self.block_device))
            .borrow().read(0, |disk_inode: &DiskInode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&mut self, inode_block_id: usize, offset: usize, buf: &[u8]) -> usize {
        let size = get_block_cache(inode_block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&mut self, inode_block_id: usize) {
        get_block_cache(inode_block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |disk_inode: &mut DiskInode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                self.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
