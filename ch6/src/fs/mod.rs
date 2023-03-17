//! File system in os
mod inode;
mod stdio;

use alloc::vec::Vec;

/// File trait
pub trait File {
    /// Read file to `UserBuffer`
    fn read(&self, buf: Vec<&'static mut [u8]>) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: Vec<&'static mut [u8]>) -> usize;
}

pub use inode::{list_apps, open_file, OSInode};
pub use stdio::{Stdin, Stdout};