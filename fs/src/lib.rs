#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use core::usize;

const BLOCK_SIZE: usize = 0x200;
const TOTAL_BLOCKS: usize = 0x20040;
const FAT_BLOCKS: usize = 0x40;
const BLOCKS_PER_CLUSTER: usize = 0x20;

/// 目录项
#[derive(Default, PartialEq, Clone)]
#[repr(C, packed)]
struct DirEntry {
    /// 文件名/文件夹名
    name: [u8; 16],
    /// 起始簇号
    cluster: usize,
    /// 文件大小（单位：字节）
    file_size: usize,
}

impl DirEntry {
    fn new(n: &str, cluster: usize, file_size: usize) -> Self {
        let mut name = [0; 16];
        name[..n.len()].copy_from_slice(n.as_bytes());
        Self {
            name,
            cluster,
            file_size,
        }
    }

    fn name(&self) -> &str {
        let len = (0..16).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }
}

pub struct File {
    dir_entry_i: usize,
    block_device: Rc<dyn BlockDevice>,
}

impl File {
    fn new(dir_entry_i: usize, block_device: Rc<dyn BlockDevice>) -> Self {
        Self {
            dir_entry_i,
            block_device,
        }
    }

    pub fn _read(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let dir_entry = self
            .block_device
            .get::<DirEntry>(FAT_BLOCKS, self.dir_entry_i);
        let file_len = dir_entry.file_size;
        let mut cluster_i = dir_entry.cluster;
        let mut start = 0;
        loop {
            for sector in 0..BLOCKS_PER_CLUSTER {
                let mut buf = [0; 0x200];
                self.block_device.read_block(
                    FAT_BLOCKS + cluster_i * BLOCKS_PER_CLUSTER + sector,
                    &mut buf,
                );
                start += 0x200;
                if start >= file_len {
                    v.extend_from_slice(&buf[..file_len % 0x200]);
                    return v;
                }
                v.extend_from_slice(&buf);
            }
            let next_cluster_i = self.block_device.get::<usize>(0, cluster_i);
            cluster_i = *next_cluster_i;
        }
    }

    pub fn _write(&self, buf: &[u8]) {
        self.block_device
            .set_closure::<DirEntry>(FAT_BLOCKS, self.dir_entry_i, |dir_entry| {
                let mut cluster_i = dir_entry.cluster;
                let mut start = 0;
                loop {
                    for sector in 0..BLOCKS_PER_CLUSTER {
                        if start >= buf.len() {
                            break;
                        }
                        let end = (start + 0x200).min(buf.len());
                        let mut block = [0; 0x200];
                        if start + 0x200 > buf.len() {
                            block[..(end % 0x200)].copy_from_slice(&buf[start..end]);
                        } else {
                            block.copy_from_slice(&buf[start..end]);
                        }
                        self.block_device.write_block(
                            FAT_BLOCKS + cluster_i * BLOCKS_PER_CLUSTER + sector,
                            &block,
                        );
                        start += 0x200;
                    }
                    if start >= buf.len() {
                        break;
                    }
                    let next_cluster_i = self.block_device.find_free_cluster();
                    self.block_device.set_cluster(next_cluster_i, usize::MAX);
                    self.block_device.set_cluster(cluster_i, next_cluster_i);
                    cluster_i = next_cluster_i;
                }

                let mut new_dir_entry: DirEntry = dir_entry.clone();
                new_dir_entry.file_size = buf.len();
                new_dir_entry
            });
    }
}

pub struct FileSystem {
    block_device: Rc<dyn BlockDevice>,
}

impl FileSystem {
    pub fn format(block_device: Rc<dyn BlockDevice>) -> Self {
        // erase
        for i in 0..TOTAL_BLOCKS {
            block_device.write_block(i, &[0; BLOCK_SIZE]);
        }
        block_device.set(0, 0, usize::MAX);
        Self { block_device }
    }

    pub fn mount(block_device: Rc<dyn BlockDevice>) -> Self {
        Self { block_device }
    }

    pub fn create(&self, name: &str) -> File {
        let cluster_i = self.block_device.find_free_cluster();
        self.block_device.set_cluster(cluster_i, usize::MAX);
        let dir_entry_i = self.block_device.find_free_dir_entry();
        self.block_device
            .set_dir_entry(dir_entry_i, DirEntry::new(name, cluster_i, 0));
        File::new(dir_entry_i, self.block_device.clone())
    }

    pub fn open(&self, name: &str, create: bool) -> Option<File> {
        if create {
            if let Some(f) = self.find(name) {
                Some(File::new(f, self.block_device.clone()))
            } else {
                let f = self.create(name);
                Some(f)
            }
        } else {
            self.find(name)
                .map(|f| File::new(f, self.block_device.clone()))
        }
    }

    pub fn ls(&self) -> Vec<String> {
        let mut v = Vec::new();
        self.block_device
            .for_each::<DirEntry>(FAT_BLOCKS, BLOCKS_PER_CLUSTER, |_, i| {
                v.push(String::from(i.name()))
            });
        v
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.block_device
            .find::<DirEntry>(FAT_BLOCKS, BLOCKS_PER_CLUSTER, |item| item.name() == name)
    }
}

pub trait BlockDevice {
    fn read_block(&self, i: usize, buf: &mut [u8]);
    fn write_block(&self, i: usize, buf: &[u8]);
}

impl dyn BlockDevice {
    fn get<'a, Item>(&self, start: usize, i: usize) -> &'a Item {
        let sector = i / (BLOCK_SIZE / size_of::<Item>());
        let offset = i % (BLOCK_SIZE / size_of::<Item>());
        let mut buf = [0; BLOCK_SIZE];
        self.read_block(start + sector, &mut buf);
        let list_len = BLOCK_SIZE / size_of::<Item>();
        let list =
            unsafe { &mut *slice_from_raw_parts_mut(buf.as_mut_ptr() as *mut Item, list_len) };
        &list[offset]
    }

    fn set_closure<Item>(&self, start: usize, i: usize, f: impl FnOnce(&Item) -> Item) {
        let sector = i / (BLOCK_SIZE / size_of::<Item>());
        let offset = i % (BLOCK_SIZE / size_of::<Item>());
        let mut buf = [0; BLOCK_SIZE];
        self.read_block(start + sector, &mut buf);
        let list_len = BLOCK_SIZE / size_of::<Item>();
        let list =
            unsafe { &mut *slice_from_raw_parts_mut(buf.as_mut_ptr() as *mut Item, list_len) };
        list[offset] = f(&list[offset]);
        self.write_block(start + sector, &buf);
    }

    fn for_each<Item>(&self, start: usize, num: usize, mut f: impl FnMut(usize, &Item)) {
        for i in 0..num {
            let mut buf = [0u8; BLOCK_SIZE];
            self.read_block(start + i, &mut buf);
            let list_len = BLOCK_SIZE / size_of::<Item>();
            let list = unsafe { &*slice_from_raw_parts(buf.as_ptr() as *const Item, list_len) };
            for j in 0..list_len {
                f(i * list_len + j, &list[j])
            }
        }
    }

    fn find<Item>(&self, start: usize, num: usize, f: impl Fn(&Item) -> bool) -> Option<usize> {
        let mut r: Option<usize> = None;
        self.for_each::<Item>(start, num, |i, item| {
            if f(item) {
                r = Some(i);
            }
        });
        return r;
    }

    fn find_free<Item: Default + PartialEq>(&self, start: usize, num: usize) -> usize {
        self.find::<Item>(start, num, |item| *item == Item::default())
            .unwrap()
    }

    fn set<Item>(&self, start: usize, i: usize, v: Item) {
        self.set_closure(start, i, |_| v);
    }

    fn find_free_cluster(&self) -> usize {
        self.find_free::<usize>(0, FAT_BLOCKS)
    }

    fn find_free_dir_entry(&self) -> usize {
        self.find_free::<DirEntry>(FAT_BLOCKS, BLOCKS_PER_CLUSTER)
    }

    fn set_dir_entry(&self, i: usize, v: DirEntry) {
        self.set::<DirEntry>(FAT_BLOCKS, i, v);
    }

    fn set_cluster(&self, i: usize, v: usize) {
        self.set::<usize>(0, i, v);
    }
}
