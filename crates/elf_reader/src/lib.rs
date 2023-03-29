#![no_std]

extern crate alloc;

mod header;

use core::ops::Range;
use alloc::vec::Vec;
use header::{Header, ProgromHeader};


pub struct ElfFile<'a> { 
    header: &'a Header,
    programs: Vec<ProgromSection<'a>>
}

impl<'a> ElfFile<'a> {
    pub fn read(data: &'a [u8]) -> Self {
        let header = unsafe {
            &*(data as *const [u8] as *const Header)
        };
        // 确保是elf格式的可执行文件
        assert_eq!(header.magic, [0x7f, b'E', b'L', b'F']);
        // 确保是64位
        assert_eq!(header.class, 0x2);

        let programs = (0..header.progrom_header_count)
            .map(|i| {
                let start_program_header = header.progrom_header_offset + i as usize * header.progrom_header_size as usize;
                let end_program_header = start_program_header + header.progrom_header_size as usize;
                let program_header = unsafe {
                    &*(&data[start_program_header..end_program_header] as *const [u8] as *const ProgromHeader)
                };
                let start_program_data = program_header.program_data_offset;
                let end_program_data = start_program_data + program_header.file_size;
                ProgromSection { 
                    header: program_header, 
                    data: &data[start_program_data..end_program_data] 
                }
            })
            .collect();
        Self { 
            header,
            programs
        }
    }

    pub fn entry_address(&self) -> usize {
        self.header.entry_address
    }

    pub fn programs(&self) -> Vec<&ProgromSection> {
        self.programs
            .iter()
            .filter(|program| {
                program.is_load()
            })
            .collect()
    }
}

pub struct ProgromSection<'a> {
    header: &'a ProgromHeader,
    pub data: &'a [u8]
}

impl<'a> ProgromSection<'_> {
    pub fn start_va(&self) -> usize {
        self.header.start_virtual_address
    }

    pub fn end_va(&self) -> usize {
        self.header.start_virtual_address + self.header.memory_size
    }

    fn is_load(&self) -> bool {
        self.header._type == 1
    }
}
