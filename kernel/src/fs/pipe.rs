use super::File;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::rc::{Rc, Weak};

use mutrc::{MutRc, MutWeak};
use page_table::PhysicalBufferList;
use crate::task::{TaskManager, suspend_and_run_next};

pub struct Pipe {
    buffer: MutRc<PipeBuffer>,
}

impl Pipe {
    pub fn new(buffer: MutRc<PipeBuffer>) -> Self {
        Self {
            buffer,
        }
    }
}

const BUFFER_SIZE: usize = 32;

pub struct PipeBuffer {
    buffer: VecDeque<u8>,
    write_end: Option<MutWeak<Pipe>>,
}

impl PipeBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &MutRc<Pipe>) {
        self.write_end = Some(write_end.downgrade());
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.buffer.push_back(byte);
    }
    pub fn read_byte(&mut self) -> u8 {
        let c = self.buffer.pop_front().unwrap();
        c
    }
    pub fn available_read(&self) -> usize {
        self.buffer.len()
    }
    pub fn available_write(&self) -> usize {
        BUFFER_SIZE - self.available_read()
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (MutRc<Pipe>, MutRc<Pipe>) {
    let mut buffer = MutRc::new(PipeBuffer::new());
    let read_end = MutRc::new(Pipe::new(buffer.clone()));
    let write_end = MutRc::new(Pipe::new(buffer.clone()));
    buffer.set_write_end(&write_end);
    (read_end, write_end)
}

impl File for Pipe {
    fn read(&mut self, buf: PhysicalBufferList) -> usize {
        let mut v = Vec::new();
        for b in buf.list {
            for bb in b {
                v.push(bb);
            }
        }
        let mut already_read = 0usize;
        loop {
            let loop_read = self.buffer.available_read();
            if loop_read == 0 {
                if self.buffer.all_write_ends_closed() {
                    return already_read;
                }
                suspend_and_run_next();
                continue;
            }
            for i in 0..loop_read {
                if i >= v.len() {
                    break;
                }
                unsafe {
                    *v[i] = self.buffer.read_byte();
                }
                already_read += 1;
            }
        }
    }
    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        let mut v = Vec::new();
        for b in buf.list {
            for bb in b {
                v.push(bb);
            }
        }
        loop {
            let loop_write = self.buffer.available_write();
            if loop_write == 0 {
                suspend_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for i in 0..loop_write {
                if i >= v.len() {
                    return  i;
                }
                let byte = &v[i];
                self.buffer.write_byte(unsafe { **byte });
            }
        }
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::PIPE
    }
}
