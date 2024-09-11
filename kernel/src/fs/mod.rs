//! File system in os
mod file;
mod pipe;
mod stdio;

/// File trait
pub trait FileInterface {
    /// Read file to `UserBuffer`
    fn read(&mut self, buf: PhysicalBufferList) -> usize;
    /// Write `UserBuffer` to file
    fn write(&mut self, buf: PhysicalBufferList) -> usize;
    fn file_type(&self) -> FileType;
}

pub enum FileType {
    STDIN,
    STDOUT,
    FILE,
    PIPE,
    TCP,
    UDP,
}

pub use file::list_apps;
use page_table::PhysicalBufferList;
pub use pipe::Pipe;
pub use stdio::{Stdin, Stdout};
