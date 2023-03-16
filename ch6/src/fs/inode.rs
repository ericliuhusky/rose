use super::File;
use crate::drivers::BLOCK_DEVICE;
use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::vec::Vec;
use fs::{EasyFileSystem, Inode};
use lazy_static::*;
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    inner: RefCell<OSInodeInner>,
}
/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Rc<Inode>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(inode: Rc<Inode>) -> Self {
        Self {
            inner: RefCell::new(OSInodeInner { offset: 0, inode }),
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.borrow_mut();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Rc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Rc::new(EasyFileSystem::root_inode(&efs))
    };
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

///Open file with flags
pub fn open_file(name: &str, create: bool) -> Option<Rc<OSInode>> {
    if create {
        if let Some(inode) = ROOT_INODE.find(name) {
            // clear size
            inode.clear();
            Some(Rc::new(OSInode::new(inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name)
                .map(|inode| Rc::new(OSInode::new(inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            Rc::new(OSInode::new(inode))
        })
    }
}

impl File for OSInode {
    fn read(&self, buf: Vec<&'static mut [u8]>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut total_read_size = 0usize;
        for slice in buf {
            let read_size = inner.inode.read_at(inner.offset, slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: Vec<&'static mut [u8]>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut total_write_size = 0usize;
        for slice in buf {
            let write_size = inner.inode.write_at(inner.offset, slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}
