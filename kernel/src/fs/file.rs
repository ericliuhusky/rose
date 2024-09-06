use super::FileInterface;
use crate::drivers::BLOCK_DEVICE;
use alloc::{string::String, vec::Vec};
use fs::{File, FileSystem};
use page_table::PhysicalBufferList;

static_var! {
    FILE_SYSTEM: FileSystem = FileSystem::mount(BLOCK_DEVICE.clone());
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    let apps: Vec<String> = FILE_SYSTEM
        .files()
        .iter()
        .map(|f| String::from(f.name()))
        .collect();
    for app in apps {
        println!("{}", app);
    }
    println!("**************/");
}

impl FileInterface for File {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        let v = self._read();
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
        self._write(&v);
        0
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::FILE
    }
}
