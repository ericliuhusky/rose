use core::ops::Range;
use alloc::vec::Vec;

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
            vpn_range: start_vpn..end_vpn,
            对齐到分页的地址范围: 对齐到分页的起始地址..对齐到分页的结尾地址,
        }
    }
    pub fn vp_list(&self) -> Vec<虚拟页> {
        let mut v = Vec::new();
        for vpn in self.vpn_range.clone() {
            v.push(虚拟页(vpn))
        }
        v
    }
    pub fn map(&self, page_table: &mut PageTable, map_type: MapType, is_user: bool) {
        for vp in self.vp_list() {
            let ppn: 物理页;
            match map_type {
                MapType::Identical => {
                    ppn = 物理页::新建(vp.0);
                }
                MapType::Framed => {
                    ppn = 物理内存管理器::分配物理页();
                }
            }
            page_table.map(vp, ppn, is_user);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}
