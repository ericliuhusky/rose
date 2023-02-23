//! Implementation of [`MapArea`] and [`MemorySet`].

use crate::mm::page_table::页表;
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

pub struct 地址空间 {
    页表: 页表,
}

impl 地址空间 {
    fn 映射(&mut self, 虚拟地址范围: Range<usize>) {
        for 虚拟页 in 逻辑段::新建(虚拟地址范围).虚拟页列表() {
            let 物理页 = 物理内存管理器::分配物理页();
            self.页表.映射(虚拟页, 物理页, false);
        }
    }
    fn 用户可见映射(&mut self, 虚拟地址范围: Range<usize>) {
        for 虚拟页 in 逻辑段::新建(虚拟地址范围).虚拟页列表() {
            let 物理页 = 物理内存管理器::分配物理页();
            self.页表.映射(虚拟页, 物理页, true);
        }
    }
    fn 恒等映射(&mut self, 虚拟地址范围: Range<usize>) {
        for 虚拟页 in 逻辑段::新建(虚拟地址范围).虚拟页列表() {
            let 物理页 = 虚拟页.clone();
            self.页表.映射(虚拟页, 物理页, false);
        }
    }

    fn 新建内核地址空间() -> Self {
        let 逻辑段范围列表 = [
            stext as usize..etext as usize,
            srodata as usize..erodata as usize,
            sdata as usize..edata as usize,
            sbss_with_stack as usize..ebss as usize,
            ekernel as usize..可用物理内存结尾地址,
            0x100000..0x102000, // MMIO VIRT_TEST/RTC  in virt machine
        ];
        let mut 地址空间 = Self { 页表: 页表::新建() };
        for 逻辑段范围 in 逻辑段范围列表 {
            地址空间.恒等映射(逻辑段范围);
        }
        // 内核栈
        地址空间.映射(内核栈栈底..内核栈栈顶);
        地址空间
    }
    
    pub fn 新建应用地址空间(elf文件数据: &[u8]) -> (页表, usize, usize) {
        let mut 地址空间 = Self { 页表: 页表::新建() };
        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        地址空间.恒等映射(__trap_entry as usize..__trap_end as usize);

        // map program headers of elf, with U flag
        let elf文件 = Elf文件::解析(elf文件数据);
        for 程序段 in elf文件.程序段列表() {
            地址空间.用户可见映射(程序段.虚拟地址范围());
            地址空间.页表.write(程序段.虚拟地址范围(), 程序段.数据);
        }

        let 最后一个程序段的虚拟地址范围 = elf文件.最后一个程序段的虚拟地址范围();
        let 用户栈栈底 = 逻辑段::新建(最后一个程序段的虚拟地址范围).结尾地址;
        let 用户栈栈顶 = 用户栈栈底 + 0x2000;
        地址空间.用户可见映射(用户栈栈底..用户栈栈顶);
        // map TrapContext
        地址空间.映射(TRAP_CONTEXT..TRAP_CONTEXT_END);
        (
            地址空间.页表,
            用户栈栈顶,
            elf文件.入口地址(),
        )
    }
    fn 激活(&self) {
        let satp = self.页表.token();
        unsafe {
            core::arch::asm!("csrw satp, {}", in(reg) satp);
            asm!("sfence.vma");
        }
    }

    pub fn 初始化内核地址空间() {
        unsafe {
            内核地址空间 = Self::新建内核地址空间();
            内核地址空间.激活();
        }
    }
    pub fn 内核地址空间token() -> usize {
        unsafe {
            内核地址空间.页表.token()
        }
    }
}

static mut 内核地址空间: 地址空间 = 地址空间 {
    页表: 页表 {
        根物理页: 内存分页 {
            页号: 0,
            起始地址: 0,
            结尾地址: 0
        },
    },
};
