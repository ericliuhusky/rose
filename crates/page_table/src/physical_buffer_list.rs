use alloc::{string::String, vec::Vec};
use core::{
    ops::{Index, IndexMut},
    str::from_utf8,
};

pub struct PhysicalBufferList {
    pub list: Vec<&'static mut [u8]>,
}

impl PhysicalBufferList {
    pub fn new(list: Vec<&'static mut [u8]>) -> Self {
        Self { list }
    }

    pub fn len(&self) -> usize {
        self.list.iter().map(|buf| buf.len()).sum()
    }

    pub fn iter(&self) -> PhysicalBufferListIterator {
        PhysicalBufferListIterator {
            list: &self.list,
            buffer_i: 0,
            byte_i: 0,
        }
    }

    pub fn iter_mut(&mut self) -> PhysicalBufferListMutIterator {
        PhysicalBufferListMutIterator {
            list: &mut self.list,
            buffer_i: 0,
            byte_i: 0,
        }
    }

    pub fn copy_from_slice(&mut self, src: &PhysicalBufferList) {
        for i in 0..self.list.len() {
            self.list[i].copy_from_slice(src.list[i]);
        }
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for buf in &self.list {
            let s = from_utf8(buf).unwrap();
            string.push_str(s);
        }
        string
    }
}

impl Index<usize> for PhysicalBufferList {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        let mut current_index = index;
        for buffer in &self.list {
            let buffer_len = buffer.len();
            if current_index < buffer_len {
                return &buffer[current_index];
            }
            current_index -= buffer_len;
        }
        panic!(
            "Index out of bounds: the len is {} but the index is {}.",
            self.len(),
            index
        );
    }
}

impl IndexMut<usize> for PhysicalBufferList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();
        let mut current_index = index;
        for buffer in &mut self.list {
            let buffer_len = buffer.len();
            if current_index < buffer_len {
                return &mut buffer[current_index];
            }
            current_index -= buffer_len;
        }
        panic!(
            "Index out of bounds: the len is {} but the index is {}.",
            len, index
        );
    }
}

impl<'a> IntoIterator for &'a PhysicalBufferList {
    type Item = u8;
    type IntoIter = PhysicalBufferListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct PhysicalBufferListIterator<'a> {
    list: &'a Vec<&'static mut [u8]>,
    buffer_i: usize,
    byte_i: usize,
}

impl<'a> Iterator for PhysicalBufferListIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_i >= self.list.len() {
            return None;
        }

        let current_buffer = &self.list[self.buffer_i];

        if self.byte_i >= current_buffer.len() {
            self.buffer_i += 1;
            self.byte_i = 0;

            return self.next();
        }

        let byte = current_buffer[self.byte_i];
        self.byte_i += 1;

        Some(byte)
    }
}

impl<'a> IntoIterator for &'a mut PhysicalBufferList {
    type Item = &'a mut u8;
    type IntoIter = PhysicalBufferListMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct PhysicalBufferListMutIterator<'a> {
    list: &'a mut Vec<&'static mut [u8]>,
    buffer_i: usize,
    byte_i: usize,
}

impl<'a> Iterator for PhysicalBufferListMutIterator<'a> {
    type Item = &'a mut u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_i >= self.list.len() {
            return None;
        }

        let current_buffer = &mut self.list[self.buffer_i];

        if self.byte_i >= current_buffer.len() {
            self.buffer_i += 1;
            self.byte_i = 0;

            return self.next();
        }

        let byte_ptr = &mut current_buffer[self.byte_i] as *mut u8;
        let byte_ref = unsafe { &mut *byte_ptr };
        self.byte_i += 1;

        Some(byte_ref)
    }
}
