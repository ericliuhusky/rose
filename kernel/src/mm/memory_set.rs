use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;
use exception::context::Context;
use super::address::{逻辑段};
use frame_allocator::FrameAllocator;
use elf_reader::ElfFile;
use lazy_static::lazy_static;

pub const MEMORY_END: usize = 0x88000000;

pub const KERNEL_STACK_START_ADDR: usize = HIGH_START_ADDR;
pub const KERNEL_STACK_END_ADDR: usize = KERNEL_STACK_START_ADDR + 0x2000;
pub const KERNEL_STACK_TOP: usize = KERNEL_STACK_END_ADDR;

extern "C" {
    fn skernel();
    fn ekernel();
    fn strampoline();
    fn etrampoline();
}

use page_table::{SV39PageTable, HIGH_START_ADDR};
use page_table::{VPN, VA};
use page_table::PageTableEntryFlags;


pub struct 地址空间 {
    pub page_table: SV39PageTable<FrameAllocator>,
    逻辑段列表: Vec<逻辑段>,
}

impl 地址空间 {
    fn 映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.虚拟页号范围() {
            let flags;
            if 逻辑段.用户可见 {
                flags = PageTableEntryFlags::UXWR;
            } else {
                flags = PageTableEntryFlags::XWR;
            }
            self.page_table.map(虚拟页号, 逻辑段.恒等映射, flags);
        }
        self.逻辑段列表.push(逻辑段);
    }

    fn 新建空地址空间() -> Self {
        Self { 
            page_table: SV39PageTable::<FrameAllocator>::new(),
            逻辑段列表: Vec::new(),
        }
    }

    pub fn 新建内核地址空间() -> Self {
        let mut 地址空间 = Self::新建空地址空间();

        地址空间.映射(逻辑段 { 
            虚拟地址范围: skernel as usize..ekernel as usize,
            恒等映射: true,
            用户可见: false,
        });
        地址空间.映射(逻辑段 { 
            虚拟地址范围: ekernel as usize..MEMORY_END,
            恒等映射: true,
            用户可见: false,
        });
        地址空间.映射(逻辑段 { 
            虚拟地址范围: 0x100000..0x102000,
            恒等映射: true,
            用户可见: false,
        }); // MMIO VIRT_TEST/RTC  in virt machine
        地址空间.映射(逻辑段 { 
            虚拟地址范围: 0x10001000..0x10002000,
            恒等映射: true,
            用户可见: false,
        }); // MMIO VIRT_TEST/RTC  in virt machine
        
        // 内核栈
        地址空间.映射(逻辑段 { 
            虚拟地址范围: KERNEL_STACK_START_ADDR..KERNEL_STACK_END_ADDR,
            恒等映射: false,
            用户可见: false,
        });
        地址空间
    }
    
    pub fn 新建应用地址空间(elf文件数据: &[u8]) -> (Self, usize) {
        let mut 地址空间 = Self::新建空地址空间();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        地址空间.映射(逻辑段 { 
            虚拟地址范围: strampoline as usize..etrampoline as usize,
            恒等映射: true,
            用户可见: false,
         });

        let elf文件 = ElfFile::read(elf文件数据);
        let 程序段列表 = elf文件.programs();
        for 程序段 in &程序段列表 {
            地址空间.映射(逻辑段 { 
                虚拟地址范围: 程序段.start_va()..程序段.end_va(),
                恒等映射: false,
                用户可见: true,
             });
            地址空间.page_table.write(程序段.start_va(), 程序段.memory_size(), 程序段.data);
        }

        地址空间.映射(逻辑段 { 
            虚拟地址范围: 0xFFFFFFFFFFFCF000..0xFFFFFFFFFFFEF000,
            恒等映射: false,
            用户可见: true,
         });

        
        (
            地址空间,
            elf文件.entry_address(),
        )
    }

    pub fn 复制地址空间(被复制的地址空间: &Self) -> Self {
        let mut 地址空间 = Self::新建空地址空间();
        for 逻辑段 in &被复制的地址空间.逻辑段列表 {
            let 虚拟地址范围 = 逻辑段.虚拟地址范围.clone();
            地址空间.映射(逻辑段 {
                虚拟地址范围: 虚拟地址范围.clone(),
                恒等映射: 逻辑段.恒等映射,
                用户可见: 逻辑段.用户可见
            });
            // TODO: 整理页表的完全复制，为何不能读完一部分数据再写入呢
            for vpn in 逻辑段.虚拟页号范围() {
                let src_ppn = 被复制的地址空间.page_table.translate(vpn).0;
                let dst_ppn = 地址空间.page_table.translate(vpn).0;
                if src_ppn == dst_ppn {
                    continue;
                }
                unsafe {
                    let dst = core::slice::from_raw_parts_mut((dst_ppn << 12) as *mut u8, 4096);
                    let src = core::slice::from_raw_parts_mut((src_ppn << 12) as *mut u8, 4096);
                    dst.copy_from_slice(src); 
                }
            }
            // let 数据 = 被复制的地址空间.读取字节数组(虚拟地址范围.clone());
            // 地址空间.多级页表.写入字节数组(虚拟地址范围.clone(), &数据);
        }
        地址空间
    }
}

impl 地址空间 {
    pub fn token(&self) -> usize {
        self.page_table.satp()
    }

    pub fn read_str(&self, va: usize, len: usize) -> String {
        self.page_table.read_str(va, len)
    }
}

lazy_static! {
    pub static ref 内核地址空间: 地址空间 = 地址空间::新建内核地址空间();
}
