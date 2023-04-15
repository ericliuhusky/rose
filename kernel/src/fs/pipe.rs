use super::File;
use crate::task::suspend_and_run_next;
use alloc::collections::VecDeque;
use mutrc::MutRc;
use page_table::PhysicalBufferList;

const PIPE_BUFFER_SIZE: usize = 32;

pub struct Pipe {
    buffer: MutRc<VecDeque<u8>>,
}

impl Pipe {
    pub fn new(buffer: MutRc<VecDeque<u8>>) -> Self {
        Self { buffer }
    }

    pub fn new_pair() -> (MutRc<Self>, MutRc<Self>) {
        let buffer = MutRc::new(VecDeque::new());
        let read_end = MutRc::new(Pipe::new(buffer.clone()));
        let write_end = MutRc::new(Pipe::new(buffer.clone()));
        (read_end, write_end)
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.buffer.push_back(byte);
    }

    pub fn read_byte(&mut self) -> u8 {
        self.buffer.pop_front().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.buffer.len() == PIPE_BUFFER_SIZE
    }
}

impl File for Pipe {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        if self.buffer.is_empty() {
            suspend_and_run_next();
        }
        let mut already_read = 0;
        for byte in &mut buf {
            if self.is_empty() {
                break;
            }
            *byte = self.read_byte();
            already_read += 1;
        }
        already_read
    }

    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        if self.is_full() {
            suspend_and_run_next();
        }
        let mut already_write = 0;
        for byte in &buf {
            if self.is_full() {
                break;
            }
            self.write_byte(byte);
            already_write += 1;
        }
        already_write
    }

    fn file_type(&self) -> super::FileType {
        super::FileType::PIPE
    }
}
