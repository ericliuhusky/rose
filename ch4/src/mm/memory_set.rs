//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::mm::page_table::PageTable;
use crate::mm::address::{将地址转为页号, 物理页, 虚拟页, 对齐到分页向下取整, 对齐到分页向上取整};
use crate::config::{可用物理内存结尾地址, MMIO, TRAP_CONTEXT, TRAP_CONTEXT_END, 内核栈栈底, 内核栈栈顶};
use alloc::vec::Vec;
use core::arch::asm;
use core::ops::Range;
use crate::mm::elf_reader::Elf文件;
use crate::格式化输出并换行;
use crate::mm::frame_allocator::物理内存管理器;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn __trap_entry();
    fn __trap_end();
}

pub static mut KERNEL_SPACE: MemorySet = MemorySet {
    page_table: PageTable {
        root_ppn: 物理页(0),
    },
    areas: Vec::new(),
};

/// memory set structure, controls virtual-memory space
pub struct MemorySet {
    pub page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    fn push(&mut self, map_area: MapArea, data: Option<&[u8]>, map_type: MapType, is_user: bool) {
        map_area.map(&mut self.page_table, map_type, is_user);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map kernel sections
        格式化输出并换行!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        格式化输出并换行!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        格式化输出并换行!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        格式化输出并换行!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        格式化输出并换行!("mapping .text section");
        memory_set.push(
            MapArea::new(stext as usize..etext as usize),
            None,
            MapType::Identical,
            false,
        );
        格式化输出并换行!("mapping .rodata section");
        memory_set.push(
            MapArea::new(srodata as usize..erodata as usize),
            None,
            MapType::Identical,
            false,
        );
        格式化输出并换行!("mapping .data section");
        memory_set.push(
            MapArea::new(sdata as usize..edata as usize),
            None,
            MapType::Identical,
            false,
        );
        格式化输出并换行!("mapping .bss section");
        memory_set.push(
            MapArea::new(sbss_with_stack as usize..ebss as usize),
            None,
            MapType::Identical,
            false,
        );
        格式化输出并换行!("mapping physical memory");
        memory_set.push(
            MapArea::new(ekernel as usize..可用物理内存结尾地址),
            None,
            MapType::Identical,
            false,
        );
        格式化输出并换行!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                MapArea::new((*pair).0..(*pair).0 + (*pair).1),
                None,
                MapType::Identical,
                false,
            );
        }
        // 内核栈
        memory_set.push(
            MapArea::new(内核栈栈底..内核栈栈顶), 
            None,
            MapType::Framed,
            false
        );

        memory_set
    }
    
    pub fn from_elf(elf_data: &[u8]) -> (PageTable, usize, usize) {
        let mut memory_set = Self::new_bare();
        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        memory_set.push(
            MapArea::new(__trap_entry as usize..__trap_end as usize),
            None,
            MapType::Identical, 
            false
        );

        // map program headers of elf, with U flag
        let elf = Elf文件::解析(elf_data);
        for p in elf.程序段列表() {
            let map_area = MapArea::new(p.虚拟地址范围());
            memory_set.push(
                map_area,
                Some(p.数据),
                MapType::Framed, 
                true
            );
        }
        let 最后一个程序段的结尾虚拟地址 = elf.最后一个程序段的结尾虚拟地址();
        let user_stack_bottom = 对齐到分页向上取整(最后一个程序段的结尾虚拟地址);
        let user_stack_top = user_stack_bottom + 0x2000;
        memory_set.push(
            MapArea::new(user_stack_bottom..user_stack_top),
            None,
            MapType::Framed,
            true,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(TRAP_CONTEXT..TRAP_CONTEXT_END),
            None,
            MapType::Framed,
            false,
        );
        (
            memory_set.page_table,
            user_stack_top,
            elf.入口地址(),
        )
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            core::arch::asm!("csrw satp, {}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
}

/// map area structure, controls a contiguous piece of virtual memory
pub struct MapArea {
    va_range: Range<usize>,
    vpn_range: Range<usize>,
    对齐到分页的地址范围: Range<usize>,
}

impl MapArea {
    pub fn new(va_range: Range<usize>) -> Self {
        let 对齐到分页的起始地址 = 对齐到分页向下取整(va_range.start);
        let 对齐到分页的结尾地址 = 对齐到分页向上取整(va_range.end);
        let start_vpn = 将地址转为页号(对齐到分页的起始地址);
        let end_vpn = 将地址转为页号(对齐到分页的结尾地址);
        crate::格式化输出并换行!("aaa {:x}..{:x}", va_range.start, va_range.end);
        crate::格式化输出并换行!("bbb {:x}..{:x}", start_vpn, end_vpn);
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
