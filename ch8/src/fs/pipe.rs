use super::File;
use core::cell::RefCell;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::rc::{Rc, Weak};

use crate::task::{TaskManager, suspend_and_run_next};

pub struct Pipe {
    buffer: Rc<RefCell<PipeBuffer>>,
}

impl Pipe {
    pub fn new(buffer: Rc<RefCell<PipeBuffer>>) -> Self {
        Self {
            buffer,
        }
    }
}

const BUFFER_SIZE: usize = 32;

pub struct PipeBuffer {
    buffer: VecDeque<u8>,
    write_end: Option<Weak<Pipe>>,
}

impl PipeBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Rc<Pipe>) {
        self.write_end = Some(Rc::downgrade(write_end));
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
pub fn make_pipe() -> (Rc<Pipe>, Rc<Pipe>) {
    let buffer = Rc::new(unsafe { RefCell::new(PipeBuffer::new()) });
    let read_end = Rc::new(Pipe::new(buffer.clone()));
    let write_end = Rc::new(Pipe::new(buffer.clone()));
    buffer.borrow_mut().set_write_end(&write_end);
    (read_end, write_end)
}

impl File for Pipe {
    fn read(&self, buf: Vec<&'static mut [u8]>) -> usize {
        let mut v = Vec::new();
        for b in buf {
            for bb in b {
                v.push(bb);
            }
        }
        let mut already_read = 0usize;
        loop {
            let mut pipe_buffer = self.buffer.borrow_mut();
            let loop_read = pipe_buffer.available_read();
            if loop_read == 0 {
                if pipe_buffer.all_write_ends_closed() {
                    return already_read;
                }
                drop(pipe_buffer);
                suspend_and_run_next();
                continue;
            }
            for i in 0..loop_read {
                if i >= v.len() {
                    break;
                }
                unsafe {
                    *v[i] = pipe_buffer.read_byte();
                }
                already_read += 1;
            }
        }
    }
    fn write(&self, buf: Vec<&'static mut [u8]>) -> usize {
        let mut v = Vec::new();
        for b in buf {
            for bb in b {
                v.push(bb);
            }
        }
        loop {
            let mut pipe_buffer = self.buffer.borrow_mut();
            let loop_write = pipe_buffer.available_write();
            if loop_write == 0 {
                drop(pipe_buffer);
                suspend_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for i in 0..loop_write {
                if i >= v.len() {
                    return  i;
                }
                let byte = &v[i];
                pipe_buffer.write_byte(unsafe { **byte });
            }
        }
    }
}
