//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::mm::page_table::PageTable;
use crate::mm::address::内存分页;
use crate::config::{可用物理内存结尾地址, MMIO, TRAP_CONTEXT, TRAP_CONTEXT_END, 内核栈栈底, 内核栈栈顶};
use core::arch::asm;
use crate::mm::elf_reader::Elf文件;
use super::map_area::MapArea;
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
        root_ppn: 内存分页 {
            页号: 0,
            对齐到分页的地址范围: 0..0
        },
    },
};

/// memory set structure, controls virtual-memory space
pub struct MemorySet {
    pub page_table: PageTable,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
        }
    }
    fn map(&mut self, map_area: MapArea, is_user: bool) {
        for vp in map_area.虚拟页列表() {
            let pp = 物理内存管理器::分配物理页();
            self.page_table.map(vp, pp, is_user);
        }
    }
    fn map_identical(&mut self, map_area: MapArea) {
        for vp in map_area.虚拟页列表() {
            let pp = vp.clone();
            self.page_table.map(vp, pp, false);
        }
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        memory_set.map_identical(
            MapArea::new(stext as usize..etext as usize),
        );
        memory_set.map_identical(
            MapArea::new(srodata as usize..erodata as usize),
        );
        memory_set.map_identical(
            MapArea::new(sdata as usize..edata as usize),
        );
        memory_set.map_identical(
            MapArea::new(sbss_with_stack as usize..ebss as usize),
        );
        memory_set.map_identical(
            MapArea::new(ekernel as usize..可用物理内存结尾地址),
        );
        for pair in MMIO {
            memory_set.map_identical(
                MapArea::new((*pair).0..(*pair).0 + (*pair).1),
            );
        }
        // 内核栈
        memory_set.map(
            MapArea::new(内核栈栈底..内核栈栈顶), 
            false
        );
        memory_set
    }
    
    pub fn from_elf(elf_data: &[u8]) -> (PageTable, usize, usize) {
        let mut memory_set = Self::new_bare();
        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        memory_set.map_identical(
            MapArea::new(__trap_entry as usize..__trap_end as usize),
        );

        // map program headers of elf, with U flag
        let elf = Elf文件::解析(elf_data);
        for p in elf.程序段列表() {
            let map_area = MapArea::new(p.虚拟地址范围());
            memory_set.map(
                map_area,
                true
            );
            memory_set.page_table.write(p.虚拟地址范围(), p.数据);
        }

        let 最后一个程序段的虚拟地址范围 = elf.最后一个程序段的虚拟地址范围();
        let user_stack_bottom = MapArea::new(最后一个程序段的虚拟地址范围).对齐到分页的地址范围.end;
        let user_stack_top = user_stack_bottom + 0x2000;
        memory_set.map(
            MapArea::new(user_stack_bottom..user_stack_top),
            true,
        );
        // map TrapContext
        memory_set.map(
            MapArea::new(TRAP_CONTEXT..TRAP_CONTEXT_END),
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
