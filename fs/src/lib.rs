#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use core::usize;

const BLOCK_SIZE: usize = 0x200;
const BLOCKS_PER_CLUSTER: usize = 0x20;

/// 记录文件系统元数据的块
#[repr(C, packed)]
struct MetaBlock {
    /// 文件分配表的块数
    fat_blocks: usize,
}

impl MetaBlock {
    fn new(fat_blocks: usize) -> Self {
        Self { fat_blocks }
    }
}

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

pub struct FileSystem {
    block_device: Rc<dyn BlockDevice>,
    fat_blocks: usize,
}

impl FileSystem {
    pub fn format(block_device: Rc<dyn BlockDevice>, size: usize) -> Self {
        let total_sectors = size / BLOCK_SIZE;
        let fat_blocks = (total_sectors - 1) / (1 + 64 * BLOCKS_PER_CLUSTER);
        // erase
        for i in 0..total_sectors {
            block_device.write_block(i, &[0; BLOCK_SIZE]);
        }
        let mb = MetaBlock::new(fat_blocks);
        block_device.set(0, 0, mb);
        block_device.set(1, 0, usize::MAX);
        Self {
            block_device,
            fat_blocks,
        }
    }

    pub fn mount(block_device: Rc<dyn BlockDevice>) -> Self {
        let mb = block_device.get::<MetaBlock>(0, 0);
        let fat_blocks = mb.fat_blocks;
        Self {
            block_device,
            fat_blocks,
        }
    }

    pub fn create(&self, name: &str) -> usize {
        let cluster_i = self.find_free_cluster();
        self.set_cluster(cluster_i, usize::MAX);
        let dir_entry_i = self.find_free_dir_entry();
        self.set_dir_entry(dir_entry_i, DirEntry::new(name, cluster_i, 0));
        dir_entry_i
    }

    pub fn write(&self, i: usize, buf: &[u8]) {
        self.block_device
            .set_closure::<DirEntry>(1 + self.fat_blocks, i, |dir_entry| {
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
                            1 + self.fat_blocks + cluster_i * BLOCKS_PER_CLUSTER + sector,
                            &block,
                        );
                        start += 0x200;
                    }
                    if start >= buf.len() {
                        break;
                    }
                    let next_cluster_i = self.find_free_cluster();
                    self.set_cluster(next_cluster_i, usize::MAX);
                    self.set_cluster(cluster_i, next_cluster_i);
                    cluster_i = next_cluster_i;
                }

                let mut new_dir_entry: DirEntry = dir_entry.clone();
                new_dir_entry.file_size = buf.len();
                new_dir_entry
            });
    }

    pub fn read(&self, i: usize) -> Vec<u8> {
        let mut v = Vec::new();
        let dir_entry = self.block_device.get::<DirEntry>(1 + self.fat_blocks, i);
        let file_len = dir_entry.file_size;
        let mut cluster_i = dir_entry.cluster;
        let mut start = 0;
        loop {
            for sector in 0..BLOCKS_PER_CLUSTER {
                let mut buf = [0; 0x200];
                self.block_device.read_block(
                    1 + self.fat_blocks + cluster_i * BLOCKS_PER_CLUSTER + sector,
                    &mut buf,
                );
                start += 0x200;
                if start >= file_len {
                    v.extend_from_slice(&buf[..file_len % 0x200]);
                    return v;
                }
                v.extend_from_slice(&buf);
            }
            let next_cluster_i = self.block_device.get::<usize>(1, cluster_i);
            cluster_i = *next_cluster_i;
        }
    }

    pub fn ls(&self) -> Vec<String> {
        let mut v = Vec::new();
        self.block_device
            .for_each::<DirEntry>(1 + self.fat_blocks, BLOCKS_PER_CLUSTER, |_, i| {
                v.push(String::from(i.name()))
            });
        v
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.block_device
            .find::<DirEntry>(1 + self.fat_blocks, BLOCKS_PER_CLUSTER, |item| {
                item.name() == name
            })
    }

    fn find_free_cluster(&self) -> usize {
        self.block_device.find_free::<usize>(1, self.fat_blocks)
    }

    fn find_free_dir_entry(&self) -> usize {
        self.block_device
            .find_free::<DirEntry>(1 + self.fat_blocks, BLOCKS_PER_CLUSTER)
    }

    fn set_dir_entry(&self, i: usize, v: DirEntry) {
        self.block_device.set::<DirEntry>(1 + self.fat_blocks, i, v);
    }

    fn set_cluster(&self, i: usize, v: usize) {
        self.block_device.set::<usize>(1, i, v);
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
}
