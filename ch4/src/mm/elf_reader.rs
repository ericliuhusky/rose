use core::ops::Range;
use alloc::vec::Vec;

pub struct ElfFile<'a> { 
    header: &'a Header,
    programs: Vec<Program<'a>>
}

impl<'a> ElfFile<'a> {
    pub fn from(data: &'a [u8]) -> Self {
        let header: &Header = read(data);
        assert_eq!(header.magic, [0x7f, b'E', b'L', b'F']);
        assert_eq!(header.class, 0x2);

        let mut programs = Vec::new();
        for i in 0..header.ph_count {
            let header = Self::program_header(data, header, i);
            let data = Self::program_data(data, header);
            programs.push(Program { header, data });
        }
        Self { 
            header,
            programs
        }
    }

    fn program_header(data: &'a [u8], header: &Header, index: u16) -> &'a ProgramHeader {
        let start = header.ph_offset + index as usize * header.ph_size as usize;
        let end = start + header.ph_size as usize;
        let data = &data[start..end];
        read(data)
    }

    fn program_data(data: &'a [u8], ph: &ProgramHeader) -> &'a [u8] {
        let start = ph.offset;
        let end = start + ph.file_size;
        &data[start..end]
    }

    pub fn entry_point(&self) -> usize {
        self.header.entry_point
    }

    pub fn programs(&self) -> Vec<&Program> {
        let mut ps = Vec::new();
        for p in &self.programs {
            if p.is_load() {
                ps.push(p);
            }
        }
        ps
    }

    pub fn last_end_va(&self) -> usize {
        let last_p = self.programs.last().unwrap();
        last_p.va_range().end
    }
}

#[repr(C)]
struct Header {
    magic: [u8; 4],
    class: u8,
    unused_placeholder1: [u8; 19],
    entry_point: usize,
    ph_offset: usize,
    unused_placeholder2: [u8; 14],
    ph_size: u16,
    ph_count: u16
}

#[repr(C)]
struct ProgramHeader {
    type_: u32,
    unused_placeholder1: [u8; 4],
    offset: usize,
    virtual_addr: usize,
    unused_placeholder2: [u8; 8],
    file_size: usize,
    mem_size: usize
}

pub struct Program<'a> {
    header: &'a ProgramHeader,
    pub data: &'a [u8]
}

impl<'a> Program<'_> {
    pub fn va_range(&self) -> Range<usize> {
        let start_va = self.header.virtual_addr;
        let end_va = start_va + self.header.mem_size;
        start_va..end_va
    }

    fn is_load(&self) -> bool {
        self.header.type_ == 0x1
    }
}

fn read<T>(input: &[u8]) -> &T {
    unsafe {
        &*(input.as_ptr() as *const T)
    }
}
