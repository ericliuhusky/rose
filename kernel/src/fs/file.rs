use super::FileInterface;
use crate::drivers::BLOCK_DEVICE;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc_ext::rc::MutRc;
use core::cell::RefCell;
use fs::FileSystem;
use lazy_static::*;
use page_table::PhysicalBufferList;
/// to implement File trait atop
pub struct File {
    dir_entry_i: usize,
}

impl File {
    pub fn new(dir_entry_i: usize) -> Self {
        Self {
            dir_entry_i,
        }
    }

    pub fn read_all(&self) -> Vec<u8> {
        FILE_SYSTEM.borrow().read(self.dir_entry_i)
    }
}

lazy_static! {
    static ref FILE_SYSTEM: RefCell<FileSystem> =
        RefCell::new(FileSystem::mount(BLOCK_DEVICE.clone()));
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
pub fn open_file(name: &str, create: bool) -> Option<MutRc<File>> {
    let fs = FILE_SYSTEM.borrow_mut();
    if create {
        if let Some(f) = fs.find(name) {
            Some(MutRc::new(File::new(f)))
        } else {
            // create file
            let f = fs.create(name);
            Some(MutRc::new(File::new(f)))
        }
    } else {
        fs.find(name).map(|f| MutRc::new(File::new(f)))
    }
}

impl FileInterface for File {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        let v = FILE_SYSTEM.borrow().read(self.dir_entry_i);
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
        FILE_SYSTEM.borrow_mut().write(self.dir_entry_i, &v);
        0
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::FILE
    }
}
