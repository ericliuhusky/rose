use crate::mm::page_table::多级页表;
use alloc::vec::Vec;
use core::ops::Range;
use crate::trap::{内核栈栈顶, 应用陷入上下文存放地址, 陷入上下文};
use core::arch::asm;
use crate::mm::elf_reader::Elf文件;
use super::address::逻辑段;
use super::page_table::页表;
use crate::mm::frame_allocator::物理内存管理器;

pub const 可用物理内存结尾地址: usize = 0x80800000;

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
    多级页表: 多级页表,
}

impl 地址空间 {
    fn 映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            let 物理页号 = 物理内存管理器::分配物理页并返回页号();
            self.多级页表.映射(虚拟页号, 物理页号, false);
        }
    }
    fn 用户可见映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            let 物理页号 = 物理内存管理器::分配物理页并返回页号();
            self.多级页表.映射(虚拟页号, 物理页号, true);
        }
    }
    fn 恒等映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            let 物理页号 = 虚拟页号;
            self.多级页表.映射(虚拟页号, 物理页号, false);
        }
    }

    fn 新建空地址空间() -> Self {
        let 物理页号 = 物理内存管理器::分配物理页并返回页号();
        Self { 
            多级页表: 多级页表 { 
                根页表: 页表 { 物理页号 } 
            }
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
        地址空间.映射(逻辑段 { 虚拟地址范围: 内核栈栈顶() - 0x2000..内核栈栈顶() });
        地址空间
    }
    
    pub fn 新建应用地址空间(elf文件数据: &[u8]) -> (Self, usize, usize) {
        let mut 地址空间 = Self::新建空地址空间();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        地址空间.恒等映射(逻辑段 { 虚拟地址范围: __trap_entry as usize..__trap_end as usize });

        let elf文件 = Elf文件::解析(elf文件数据);
        let 程序段列表 = elf文件.程序段列表();
        for 程序段 in &程序段列表 {
            地址空间.用户可见映射(逻辑段 { 虚拟地址范围: 程序段.虚拟地址范围() });
            地址空间.多级页表.write(程序段.虚拟地址范围(), 程序段.数据);
        }

        let 最后一个程序段的虚拟地址范围 = 程序段列表.last().unwrap().虚拟地址范围();

        let 用户栈栈底 = 逻辑段 { 虚拟地址范围: 最后一个程序段的虚拟地址范围 }.对齐到分页的结尾地址();
        let 用户栈栈顶 = 用户栈栈底 + 0x2000;
        地址空间.用户可见映射(逻辑段 { 虚拟地址范围: 用户栈栈底..用户栈栈顶 });

        地址空间.映射(逻辑段 { 虚拟地址范围: 应用陷入上下文存放地址()..0xfffffffffffff000 });
        
        (
            地址空间,
            用户栈栈顶,
            elf文件.入口地址(),
        )
    }
}

impl 地址空间 {
    pub fn 陷入上下文(&self) -> &'static mut 陷入上下文 {
        let pa_ranges = self.多级页表.虚拟地址范围转换物理地址范围列表(应用陷入上下文存放地址()..0xfffffffffffff000);
        unsafe {
            &mut *(pa_ranges[0].start as *mut 陷入上下文)
        }
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.多级页表.根页表.物理页号
    }

    pub fn 读取字节数组(&self, 虚拟地址范围: Range<usize>) -> Vec<u8> {
        self.多级页表.读取字节数组(虚拟地址范围)
    }

    pub fn 切换到当前地址空间(&self) {
        let satp = self.token();
        unsafe {
            asm!("csrw satp, {}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
}

pub static mut 内核地址空间: 地址空间 = 地址空间 {
    多级页表: 多级页表 {
        根页表: 页表 {
            物理页号: 0
        }
    },
};
