//! File system in os
mod inode;
mod stdio;
mod pipe;

use alloc::vec::Vec;

/// File trait
pub trait File {
    /// Read file to `UserBuffer`
    fn read(&mut self, buf: Vec<&'static mut [u8]>) -> usize;
    /// Write `UserBuffer` to file
    fn write(&mut self, buf: Vec<&'static mut [u8]>) -> usize;
}

pub use inode::{list_apps, open_file, OSInode};
pub use stdio::{Stdin, Stdout};
pub use pipe::make_pipe;
