use alloc::vec::Vec;

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

    pub fn copy_from_slice(&mut self, src: &PhysicalBufferList) {
        for i in 0..self.list.len() {
            self.list[i].copy_from_slice(src.list[i]);
        }
    }
}

impl IntoIterator for PhysicalBufferList {
    type Item = u8;
    type IntoIter = PhysicalBufferListIterator;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalBufferListIterator {
            list: self.list,
            buffer_i: 0,
            byte_i: 0,
        }
    }
}

pub struct PhysicalBufferListIterator {
    list: Vec<&'static mut [u8]>,
    buffer_i: usize,
    byte_i: usize,
}

impl Iterator for PhysicalBufferListIterator {
    type Item = u8;

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

        let byte = current_buffer[self.byte_i];
        self.byte_i += 1;

        Some(byte)
    }
}

impl<'a> IntoIterator for &'a mut PhysicalBufferList {
    type Item = &'a mut u8;
    type IntoIter = PhysicalBufferListMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalBufferListMutIterator {
            list: &mut self.list,
            buffer_i: 0,
            byte_i: 0,
        }
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
