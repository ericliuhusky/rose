//! File system in os
mod inode;
mod stdio;
mod pipe;


/// File trait
pub trait File {
    /// Read file to `UserBuffer`
    fn read(&mut self, buf: PhysicalBufferList) -> usize;
    /// Write `UserBuffer` to file
    fn write(&mut self, buf: PhysicalBufferList) -> usize;
    fn file_type(&self) -> FileType;
}

pub enum FileType {
    STDIN,
    STDOUT,
    INODE,
    PIPE,
    TCP,
    UDP,
}

pub use inode::{list_apps, open_file, OSInode};
use page_table::PhysicalBufferList;
pub use stdio::{Stdin, Stdout};
pub use pipe::make_pipe;
