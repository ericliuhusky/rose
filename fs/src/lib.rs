#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

// 128MiB: 256 files, 0.5MiB each file
const BLOCK_SIZE: usize = 0x200;
const TOTAL_BLOCKS: usize = 0x40000;
const BLOCKS_PER_FILE: usize = 0x400;
const DIR_BLOCKS: usize = 0x10;

/// 目录项
#[derive(Default, PartialEq, Clone)]
#[repr(C, packed)]
struct DirEntry {
    /// 文件名
    name: [u8; 24],
    /// 文件大小（单位：字节）
    file_size: usize,
}

#[derive(Clone)]
pub struct File {
    i: usize,
    dir_entry: DirEntry,
}

impl File {
    fn new(i: usize, dir_entry: DirEntry) -> Self {
        Self { i, dir_entry }
    }

    fn is_empty(&self) -> bool {
        self.dir_entry == DirEntry::default()
    }

    pub fn name(&self) -> &str {
        let len = (0..24).find(|i| self.dir_entry.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.dir_entry.name[..len]).unwrap()
    }

    fn set_name(&mut self, n: &str) {
        let mut name = [0; 24];
        name[..n.len()].copy_from_slice(n.as_bytes());
        self.dir_entry.name = name;
    }

    pub fn _read(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let block_num = self.dir_entry.file_size / BLOCK_SIZE;
        let remainder = self.dir_entry.file_size % BLOCK_SIZE;
        let mut buf = [0; BLOCK_SIZE];
        for i in 0..block_num {
            block_device().read_block(DIR_BLOCKS + self.i * BLOCKS_PER_FILE + i, &mut buf);
            v.extend_from_slice(&buf);
        }
        block_device().read_block(DIR_BLOCKS + self.i * BLOCKS_PER_FILE + block_num, &mut buf);
        v.extend_from_slice(&buf[..remainder]);
        v
    }

    pub fn _write(&mut self, buf: &[u8]) {
        let block_num = buf.len() / BLOCK_SIZE;
        let remainder = buf.len() % BLOCK_SIZE;
        let mut block = [0; BLOCK_SIZE];
        for i in 0..block_num {
            block.copy_from_slice(&buf[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]);
            block_device().write_block(DIR_BLOCKS + self.i * BLOCKS_PER_FILE + i, &block);
        }
        block[..remainder].copy_from_slice(&buf[block_num * BLOCK_SIZE..]);
        block_device().write_block(DIR_BLOCKS + self.i * BLOCKS_PER_FILE + block_num, &block);

        self.dir_entry.file_size = buf.len();
        self.set_dir_entry();
    }

    fn set_dir_entry(&self) {
        let block = self.i / (BLOCK_SIZE / size_of::<DirEntry>());
        let offset = self.i % (BLOCK_SIZE / size_of::<DirEntry>());
        let mut buf = [0; BLOCK_SIZE];
        block_device().read_block(block, &mut buf);
        let list_len = BLOCK_SIZE / size_of::<DirEntry>();
        let list =
            unsafe { &mut *slice_from_raw_parts_mut(buf.as_mut_ptr() as *mut DirEntry, list_len) };
        list[offset] = self.dir_entry.clone();
        block_device().write_block(block, &buf);
    }
}

pub fn open(name: &str, is_create: bool) -> Option<File> {
    if is_create {
        if let Some(f) = find(name) {
            Some(f)
        } else {
            let f = create(name);
            Some(f)
        }
    } else {
        find(name)
    }
}

fn create(name: &str) -> File {
    let mut f = files().iter().find(|f| f.is_empty()).unwrap().clone();
    f.set_name(name);
    f.set_dir_entry();
    f
}

fn find(name: &str) -> Option<File> {
    files().iter().find(|f| f.name() == name).cloned()
}

pub fn files() -> Vec<File> {
    let mut v = Vec::new();
    for i in 0..DIR_BLOCKS {
        let mut buf = [0; BLOCK_SIZE];
        block_device().read_block(i, &mut buf);
        let list_len = BLOCK_SIZE / size_of::<DirEntry>();
        let list = unsafe { &*slice_from_raw_parts(buf.as_ptr() as *const DirEntry, list_len) };
        for (j, item) in list.iter().enumerate() {
            v.push(File::new(i * list_len + j, item.clone()));
        }
    }
    v
}

pub fn format() {
    // erase
    for i in 0..TOTAL_BLOCKS {
        block_device().write_block(i, &[0; BLOCK_SIZE]);
    }
}

pub trait BlockDevice {
    fn read_block(&self, i: usize, buf: &mut [u8]);
    fn write_block(&self, i: usize, buf: &[u8]);
}

static mut BLOCK_DEVICE: Option<&'static dyn BlockDevice> = None;

fn block_device() -> &'static dyn BlockDevice {
    unsafe { BLOCK_DEVICE }.unwrap()
}

pub fn init(block_device: &'static dyn BlockDevice) {
    unsafe {
        BLOCK_DEVICE = Some(block_device);
    }
}
