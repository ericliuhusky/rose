use super::File;
use crate::drivers::BLOCK_DEVICE;
use alloc_ext::rc::MutRc;
use page_table::PhysicalBufferList;
use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::vec::Vec;
use fs::FileSystem;
use lazy_static::*;
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    inode: usize,
    offset: RefCell<usize>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(inode: usize) -> Self {
        Self {
            inode,
            offset: RefCell::new(0),
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut offset = self.offset.borrow_mut();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = FILE_SYSTEM.borrow().read_at(self.inode, *offset, &mut buffer);
            if len == 0 {
                break;
            }
            *offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    static ref FILE_SYSTEM: RefCell<FileSystem> = RefCell::new(FileSystem::open(BLOCK_DEVICE.clone()));
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in FILE_SYSTEM.borrow().ls() {
        println!("{}", app);
    }
    println!("**************/");
}

///Open file with flags
pub fn open_file(name: &str, create: bool) -> Option<MutRc<OSInode>> {
    let mut fs = FILE_SYSTEM.borrow_mut();
    if create {
        if let Some(inode) = fs.find(name) {
            fs.clear(inode);
            Some(MutRc::new(OSInode::new(inode)))
        } else {
            // create file
            fs
                .create_inode(name)
                .map(|inode| MutRc::new(OSInode::new(inode)))
        }
    } else {
        fs.find(name).map(|inode| {
            MutRc::new(OSInode::new(inode))
        })
    }
}

impl File for OSInode {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        let mut offset = self.offset.borrow_mut();
        let mut total_read_size = 0usize;
        for slice in buf.list {
            let read_size = FILE_SYSTEM.borrow().read_at(self.inode, *offset, slice);
            if read_size == 0 {
                break;
            }
            *offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        let mut offset = self.offset.borrow_mut();
        let mut total_write_size = 0usize;
        for slice in buf.list {
            let write_size = FILE_SYSTEM.borrow_mut().write_at(self.inode, *offset, slice);
            assert_eq!(write_size, slice.len());
            *offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::INODE    
    }
}
