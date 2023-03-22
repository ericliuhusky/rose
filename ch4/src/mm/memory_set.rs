use alloc::vec::Vec;
use core::ops::Range;
use exception::context::Context;
use elf_reader::ElfFile;
use crate::mm::frame_allocator::物理内存管理器;
use lazy_static::lazy_static;

pub const 可用物理内存结尾地址: usize = 0x80800000;

#[no_mangle]
#[link_section = ".text.trampoline"]
static KERNEL_STACK_TOP: usize = 0xfffffffffffff000;
#[link_section = ".text.trampoline"]
#[no_mangle]
static CONTEXT_START_ADDR: usize = 0xffffffffffffe000;


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
    fn etrampoline();
}

use page_table::SV39PageTable;
use page_table::{VPN, VA};
use page_table::PageTableEntryFlags;


pub struct 地址空间 {
    page_table: SV39PageTable<物理内存管理器>
}

impl 地址空间 {
    fn 映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            self.page_table.map(VPN::new(虚拟页号), false, PageTableEntryFlags::XWR);
        }
    }
    fn 用户可见映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            self.page_table.map(VPN::new(虚拟页号), false, PageTableEntryFlags::UXWR);
        }
    }
    fn 恒等映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            self.page_table.map(VPN::new(虚拟页号), true, PageTableEntryFlags::XWR);
        }
    }

    fn 新建空地址空间() -> Self {
        Self { 
            page_table: SV39PageTable::<物理内存管理器>::new()
        }
    }

    pub fn 新建内核地址空间() -> Self {
        let mut 地址空间 = Self::新建空地址空间();

        地址空间.恒等映射(逻辑段 { 虚拟地址范围: stext as usize..etext as usize });
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: srodata as usize..erodata as usize });
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: sdata as usize..edata as usize });
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: sbss_with_stack as usize..ebss as usize });
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: ekernel as usize..可用物理内存结尾地址 });
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: 0x100000..0x102000 }); // MMIO VIRT_TEST/RTC  in virt machine
        // 内核栈
        地址空间.映射(逻辑段 { 虚拟地址范围: KERNEL_STACK_TOP - 0x2000..KERNEL_STACK_TOP });
        地址空间
    }
    
    pub fn 新建应用地址空间(elf文件数据: &[u8]) -> (Self, usize, usize) {
        let mut 地址空间 = Self::新建空地址空间();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: strampoline as usize..etrampoline as usize });

        let elf文件 = ElfFile::read(elf文件数据);
        let 程序段列表 = elf文件.programs();
        for 程序段 in &程序段列表 {
            地址空间.用户可见映射(逻辑段 { 虚拟地址范围: 程序段.virtual_address_range() });
            地址空间.page_table.write(VA::new(程序段.virtual_address_range().start), VA::new(程序段.virtual_address_range().end), 程序段.data)
        }

        let 最后一个程序段的虚拟地址范围 = 程序段列表.last().unwrap().virtual_address_range();

        let 用户栈栈底 = 逻辑段 { 虚拟地址范围: 最后一个程序段的虚拟地址范围 }.对齐到分页的结尾地址();
        let 用户栈栈顶 = 用户栈栈底 + 0x2000;
        地址空间.用户可见映射(逻辑段 { 虚拟地址范围: 用户栈栈底..用户栈栈顶 });

        地址空间.映射(逻辑段 { 虚拟地址范围: CONTEXT_START_ADDR..0xfffffffffffff000 });
        
        (
            地址空间,
            用户栈栈顶,
            elf文件.entry_address(),
        )
    }
}

impl 地址空间 {
    pub fn 陷入上下文(&self) -> &'static mut Context {
        self.page_table.translate_type(CONTEXT_START_ADDR)
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.page_table.root_ppn.0
    }

    pub fn 读取字节数组(&self, 虚拟地址范围: Range<usize>) -> Vec<u8> {
        self.page_table.read(VA::new(虚拟地址范围.start), VA::new(虚拟地址范围.end))
    }
}

lazy_static! {
    pub static ref 内核地址空间: 地址空间 = 地址空间::新建内核地址空间();
}



/// 一段连续地址的虚拟内存
pub struct 逻辑段 {
    pub 虚拟地址范围: Range<usize>
}

impl 逻辑段 {
    pub fn 虚拟页号范围(&self) -> Range<usize> {
        let 起始页号 = VA::new(self.虚拟地址范围.start).align_to_lower().page_number().0;
        let 结尾页号 = VA::new(self.虚拟地址范围.end).align_to_upper().page_number().0;
        起始页号..结尾页号
    }

    pub fn 对齐到分页的结尾地址(&self) -> usize {
        VA::new(self.虚拟地址范围.end).align_to_upper().0
    }
}
