#[repr(C)]
pub struct Header {
    pub magic: [u8; 4],
    pub class: u8,
    _unused_placeholder_1: [u8; 19],
    pub entry_address: usize,
    pub progrom_header_offset: usize,
    _unused_placeholder_2: [u8; 14],
    pub progrom_header_size: u16,
    pub progrom_header_count: u16
}

#[repr(C)]
pub struct ProgromHeader {
    pub _type: u32,
    _unused_placeholder_1: [u8; 4],
    pub program_data_offset: usize,
    pub start_virtual_address: usize,
    _unused_placeholder_2: [u8; 8],
    pub file_size: usize,
    pub memory_size: usize
}
