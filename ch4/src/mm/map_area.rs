use core::ops::Range;
use crate::mm::frame_allocator::物理内存管理器;
use crate::mm::address::{物理页, 虚拟页};
use crate::mm::page_table::PageTable;

fn 将地址转为页号(地址: usize) -> usize {
    地址 >> 12
}
fn 对齐到分页向下取整(地址: usize) -> usize {
    地址 & !0xfff
}
fn 对齐到分页向上取整(地址: usize) -> usize {
    (地址 + 0xfff) & !0xfff
}

/// map area structure, controls a contiguous piece of virtual memory
pub struct MapArea {
    va_range: Range<usize>,
    pub vpn_range: Range<usize>,
    pub 对齐到分页的地址范围: Range<usize>,
}

impl MapArea {
    pub fn new(va_range: Range<usize>) -> Self {
        let 对齐到分页的起始地址 = 对齐到分页向下取整(va_range.start);
        let 对齐到分页的结尾地址 = 对齐到分页向上取整(va_range.end);
        let start_vpn = 将地址转为页号(对齐到分页的起始地址);
        let end_vpn = 将地址转为页号(对齐到分页的结尾地址);
        Self {
            va_range,
            vpn_range: start_vpn..end_vpn,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址,
        }
    }
    pub fn 新建内嵌于地址范围的逻辑段(va_range: Range<usize>) -> Self {
        let 对齐到分页的起始地址 = 对齐到分页向上取整(va_range.start);
        let 对齐到分页的结尾地址 = 对齐到分页向下取整(va_range.end);
        let start_vpn = 将地址转为页号(对齐到分页的起始地址);
        let end_vpn = 将地址转为页号(对齐到分页的结尾地址);
        Self {
            va_range,
            vpn_range: start_vpn..end_vpn,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址,
        }
    }
    pub fn map(&self, page_table: &mut PageTable, map_type: MapType, is_user: bool) {
        for vpn in self.vpn_range.clone() {
            let ppn: 物理页;
            match map_type {
                MapType::Identical => {
                    ppn = 物理页(vpn);
                }
                MapType::Framed => {
                    ppn = 物理内存管理器::分配物理页();
                }
            }
            page_table.map(虚拟页(vpn), ppn, is_user);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&self, page_table: &mut PageTable, data: &[u8]) {
        let dsts = page_table.translated_byte_buffer(self.va_range.clone());
        let mut i = 0;
        for dst in dsts {
            if i >= data.len() {
                break;
            }
            let src = &data[i..i + dst.len()];
            i += dst.len();
            for i in 0..dst.len() {
                dst[i] = src[i];
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}
