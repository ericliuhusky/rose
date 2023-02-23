//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::mm::page_table::PageTable;
use crate::mm::address::内存分页;
use crate::config::{可用物理内存结尾地址, TRAP_CONTEXT, TRAP_CONTEXT_END, 内核栈栈底, 内核栈栈顶};
use core::arch::asm;
use core::ops::Range;
use crate::mm::elf_reader::Elf文件;
use super::map_area::逻辑段;
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
            起始地址: 0,
            结尾地址: 0
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
    fn 映射(&mut self, va_range: Range<usize>) {
        for vp in 逻辑段::新建(va_range).虚拟页列表() {
            let pp = 物理内存管理器::分配物理页();
            self.page_table.map(vp, pp, false);
        }
    }
    fn 用户可见映射(&mut self, va_range: Range<usize>) {
        for vp in 逻辑段::新建(va_range).虚拟页列表() {
            let pp = 物理内存管理器::分配物理页();
            self.page_table.map(vp, pp, true);
        }
    }
    fn 恒等映射(&mut self, va_range: Range<usize>) {
        for vp in 逻辑段::新建(va_range).虚拟页列表() {
            let pp = vp.clone();
            self.page_table.map(vp, pp, false);
        }
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let segment_ranges = [
            stext as usize..etext as usize,
            srodata as usize..erodata as usize,
            sdata as usize..edata as usize,
            sbss_with_stack as usize..ebss as usize,
            ekernel as usize..可用物理内存结尾地址,
            0x100000..0x102000, // MMIO VIRT_TEST/RTC  in virt machine
        ];
        let mut memory_set = Self::new_bare();
        for segment_range in segment_ranges {
            memory_set.恒等映射(segment_range);
        }
        // 内核栈
        memory_set.映射(内核栈栈底..内核栈栈顶);
        memory_set
    }
    
    pub fn from_elf(elf_data: &[u8]) -> (PageTable, usize, usize) {
        let mut memory_set = Self::new_bare();
        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        memory_set.恒等映射(__trap_entry as usize..__trap_end as usize);

        // map program headers of elf, with U flag
        let elf = Elf文件::解析(elf_data);
        for p in elf.程序段列表() {
            memory_set.用户可见映射(p.虚拟地址范围());
            memory_set.page_table.write(p.虚拟地址范围(), p.数据);
        }

        let 最后一个程序段的虚拟地址范围 = elf.最后一个程序段的虚拟地址范围();
        let user_stack_bottom = 逻辑段::新建(最后一个程序段的虚拟地址范围).结尾地址;
        let user_stack_top = user_stack_bottom + 0x2000;
        memory_set.用户可见映射(user_stack_bottom..user_stack_top);
        // map TrapContext
        memory_set.映射(TRAP_CONTEXT..TRAP_CONTEXT_END);
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
