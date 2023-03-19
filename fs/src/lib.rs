//!An file system isolated from the kernel
#![no_std]
extern crate alloc;
mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod layout;
mod vfs;
/// Use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::FileSystem;
use layout::*;
pub use vfs::Inode;
pub mod lru;

/*
SuperBlock          1
InodeBitmapBlock    1
InodeAreaBlock      64
DataBitmapBlock     8
DataAreaBlock       8 * 4096
*/
const INODE_BITMAP_BLOCK_NUM: u32 = 1;
const INODE_AREA_BLOCK_NUM: u32 = 64;
const DATA_BITMAP_BLOCK_NUM: u32 = 8;
const DATA_AREA_BLOCK_NUM: u32 = DATA_BITMAP_BLOCK_NUM * 4096;
pub const TOTAL_BLOCK_NUM: u32 = 1 + INODE_BITMAP_BLOCK_NUM + INODE_AREA_BLOCK_NUM + DATA_BITMAP_BLOCK_NUM + DATA_AREA_BLOCK_NUM;
