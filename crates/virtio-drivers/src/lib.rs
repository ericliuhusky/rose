#![no_std]

extern crate alloc;
extern crate core_ext;

mod blk;
mod hal;
mod header;
mod net;
mod queue;
mod volatile;

pub use self::blk::{BlkResp, RespStatus, VirtIOBlk};
pub use self::hal::{Hal, PhysAddr, VirtAddr};
pub use self::header::*;
pub use self::net::VirtIONet;
use self::queue::VirtQueue;
use core::mem::size_of;
use core_ext::UInt;
use hal::*;

/// The page size in bytes supported by the library (4 KiB).
const PAGE_SIZE: usize = 0x1000;

/// Align `size` up to a page.
fn align_up(size: usize) -> usize {
    UInt(size).align_to_upper(PAGE_SIZE)
}

/// Convert a struct into a byte buffer.
unsafe trait AsBuf: Sized {
    fn as_buf(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as _, size_of::<Self>()) }
    }
    fn as_buf_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as _, size_of::<Self>()) }
    }
}
