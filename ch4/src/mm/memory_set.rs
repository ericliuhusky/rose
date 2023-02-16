//! Implementation of [`MapArea`] and [`MemorySet`].

use super::{frame_alloc};
use super::{PageTable};
use super::{PhysPageNum, VirtPageNum};
use crate::mm::address::{floor, ceil};
use crate::config::{MEMORY_END, MMIO, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use alloc::vec::Vec;
use core::arch::asm;
use core::ops::Range;
use crate::mm::elf_reader::ElfFile;

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
    fn strampoline();
}

pub static mut KERNEL_SPACE: MemorySet = MemorySet {
    page_table: PageTable {
        root_ppn: PhysPageNum(0),
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
    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        va_range: Range<usize>,
        is_user: bool,
    ) {
        self.push(
            MapArea::new(va_range, MapType::Framed, is_user),
            None,
        );
    }
    fn push(&mut self, map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&self) {
        self.page_table.map(
            VirtPageNum::from(TRAMPOLINE),
            PhysPageNum::from(strampoline as usize),
            false,
        );
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                stext as usize..etext as usize,
                MapType::Identical,
                false,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                srodata as usize..erodata as usize,
                MapType::Identical,
                false,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                sdata as usize..edata as usize,
                MapType::Identical,
                false,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                sbss_with_stack as usize..ebss as usize,
                MapType::Identical,
                false,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                ekernel as usize..MEMORY_END,
                MapType::Identical,
                false,
            ),
            None,
        );
        println!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                MapArea::new(
                    (*pair).0..(*pair).0 + (*pair).1,
                    MapType::Identical,
                    false,
                ),
                None,
            );
        }
        memory_set
    }
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (PageTable, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = ElfFile::from(elf_data);
        for p in elf.programs() {
            let map_area = MapArea::new(p.va_range(), MapType::Framed, true);
            memory_set.push(
                map_area,
                Some(p.data)
            );
        }
        let last_end_va = elf.last_end_va();
        let mut user_stack_bottom = last_end_va;
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom..user_stack_top,
                MapType::Framed,
                true,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT..TRAMPOLINE,
                MapType::Framed,
                false,
            ),
            None,
        );
        (
            memory_set.page_table,
            user_stack_top,
            elf.entry_point(),
        )
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            // core::arch::asm!("csrrw x0, {1}, {0}", in(reg) bits, 0x180)
            core::arch::asm!("csrw satp, {}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
}

/// map area structure, controls a contiguous piece of virtual memory
pub struct MapArea {
    va_range: Range<usize>,
    vpn_range: Range<usize>,
    map_type: MapType,
    is_user: bool,
}

impl MapArea {
    pub fn new(
        va_range: Range<usize>,
        map_type: MapType,
        is_user: bool,
    ) -> Self {
        let start_vpn = floor(va_range.start);
        let end_vpn = ceil(va_range.end);
        Self {
            va_range,
            vpn_range: start_vpn..end_vpn,
            map_type,
            is_user,
        }
    }
    pub fn map(&self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.clone() {
            let ppn: PhysPageNum;
            match self.map_type {
                MapType::Identical => {
                    ppn = PhysPageNum(vpn);
                }
                MapType::Framed => {
                    ppn = frame_alloc();
                }
            }
            page_table.map(VirtPageNum(vpn), ppn, self.is_user);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let dsts = page_table.translated_byte_buffer(self.va_range.clone());
        let mut i = 0;
        for dst in dsts {
            if i >= data.len() {
                break;
            }
            let src = &data[i..i + dst.len()];
            i += dst.len();
            dst.copy_from_slice(src);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}
