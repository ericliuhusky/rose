//!Stdin & Stdout
use super::File;
use alloc::vec::Vec;
use page_table::PhysicalBufferList;
use crate::task::{TaskManager, suspend_and_run_next};
///Standard input
pub struct Stdin;
///Standard output
pub struct Stdout;

impl File for Stdin {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        assert_eq!(buf.len(), 1);
        // busy loop
        let mut c: usize;
        loop {
            c = sbi_call::getchar();
            if c == 0 {
                suspend_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            buf.list[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        panic!("Cannot write to stdin!");
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::STDIN
    }
}

impl File for Stdout {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        for buffer in &buf.list {
            print!("{}", core::str::from_utf8(buffer).unwrap());
        }
        buf.len()
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::STDOUT
    }
}
