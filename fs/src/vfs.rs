use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    FileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::{RefCell, RefMut};
/// Virtual filesystem layer over fs
pub struct Inode {
    block_id: usize,
    fs: Rc<RefCell<FileSystem>>,
    block_device: Rc<dyn BlockDevice>,
}

impl Inode {
    /// Create a vfs inode
    pub fn new(
        block_id: u32,
        fs: Rc<RefCell<FileSystem>>,
        block_device: Rc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            fs,
            block_device,
        }
    }
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
    pub fn find(&self, name: &str) -> Option<Rc<Inode>> {
        let fs = self.fs.borrow();
        get_block_cache(self.block_id, Rc::clone(&self.block_device))
            .borrow()
            .read(0, |disk_inode| {
                self.find_inode_id(name, disk_inode).map(|inode_id| {
                    let block_id = fs.get_inode_block_id(inode_id);
                    Rc::new(Self::new(
                        block_id,
                        self.fs.clone(),
                        self.block_device.clone(),
                    ))
                })
            })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut RefMut<FileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// Create inode under current inode by name
    pub fn create(&self, name: &str) -> Option<Rc<Inode>> {
        let mut fs = self.fs.borrow_mut();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if get_block_cache(self.block_id, Rc::clone(&self.block_device)).borrow().read(0, op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        let new_inode_block_id = fs.get_inode_block_id(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
            .set(0, DiskInode::new(DiskInodeType::File));
        
        get_block_cache(self.block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |root_inode: &mut DiskInode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let block_id = fs.get_inode_block_id(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Rc::new(Self::new(
            block_id,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.borrow();
        get_block_cache(self.block_id, Rc::clone(&self.block_device))
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
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.borrow();
        get_block_cache(self.block_id, Rc::clone(&self.block_device))
            .borrow().read(0, |disk_inode: &DiskInode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.borrow_mut();
        let size = get_block_cache(self.block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.borrow_mut();
        get_block_cache(self.block_id, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(0, |disk_inode: &mut DiskInode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
