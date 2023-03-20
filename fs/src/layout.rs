/// The max length of inode name
const NAME_LENGTH_LIMIT: usize = 27;

/// Super block of a filesystem
#[repr(C)]
pub struct SuperBlock {
    pub inode_bitmap_block_num: u32,
    pub inode_area_block_num: u32,
    pub data_bitmap_block_num: u32,
    pub data_area_block_num: u32,
}

impl SuperBlock {
    pub fn new(
        inode_bitmap_block_num: u32,
        inode_area_block_num: u32,
        data_bitmap_block_num: u32,
        data_area_block_num: u32,
    ) -> Self {
        Self {
            inode_bitmap_block_num,
            inode_area_block_num,
            data_bitmap_block_num,
            data_area_block_num,
        }
    }
}

/// A directory entry
#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT + 1],
    inode_number: u32,
}
/// Size of a directory entry
pub const DIRENT_SZ: usize = 32;

impl DirEntry {
    /// Create an empty directory entry
    pub fn empty() -> Self {
        Self {
            name: [0u8; NAME_LENGTH_LIMIT + 1],
            inode_number: 0,
        }
    }
    /// Crate a directory entry from name and inode number
    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut bytes = [0u8; NAME_LENGTH_LIMIT + 1];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            inode_number,
        }
    }
    /// Serialize into bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIRENT_SZ) }
    }
    /// Serialize into mutable bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIRENT_SZ) }
    }
    /// Get name of the entry
    pub fn name(&self) -> &str {
        let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }
    /// Get inode number of the entry
    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }
}
