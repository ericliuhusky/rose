use super::File;
use crate::drivers::BLOCK_DEVICE;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc_ext::rc::MutRc;
use core::cell::RefCell;
use fs::FileSystem;
use lazy_static::*;
use page_table::PhysicalBufferList;
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
        FILE_SYSTEM.borrow().read(self.inode)
    }
}

lazy_static! {
    static ref FILE_SYSTEM: RefCell<FileSystem> =
        RefCell::new(FileSystem::open(BLOCK_DEVICE.clone()));
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
    let fs = FILE_SYSTEM.borrow_mut();
    if create {
        if let Some(inode) = fs.find(name) {
            Some(MutRc::new(OSInode::new(inode)))
        } else {
            // create file
            let inode = fs.create(name);
            Some(MutRc::new(OSInode::new(inode)))
        }
    } else {
        fs.find(name).map(|inode| MutRc::new(OSInode::new(inode)))
    }
}

impl File for OSInode {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        let v = FILE_SYSTEM.borrow().read(self.inode);
        let mut start = 0;
        for slice in buf.list {
            let end = (start + slice.len()).min(v.len());
            slice[..v.len()].copy_from_slice(&v[start..end]);
            start += slice.len();
        }
        v.len()
    }
    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        let mut v = Vec::new();
        for slice in buf.list {
            v.extend_from_slice(slice);
        }
        FILE_SYSTEM.borrow_mut().write(self.inode, &v);
        0
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::INODE
    }
}
