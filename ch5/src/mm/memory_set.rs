use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;
use crate::trap::{内核栈栈顶, 应用陷入上下文存放地址, 陷入上下文};
use crate::mm::elf_reader::Elf文件;
use super::address::{逻辑段, 连续地址虚拟内存};
// use super::frame_allocator::物理帧;
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

use crate::page_table::SV39PageTable;
use crate::page_table::FrameAlloc;
use crate::page_table::{PPN, VPN, VA};
use crate::page_table::PageTableEntryFlags;

struct TT;
impl FrameAlloc for TT {
    fn alloc() -> PPN {
        let n = 物理内存管理器::分配物理页并返回页号();
        PPN::new(n)
    }

    fn dealloc(frame: PPN) {
        物理内存管理器::回收物理帧(frame);
    }
}

pub struct 地址空间 {
    page_table: SV39PageTable<TT>,
    frames: Vec<PPN>,
    逻辑段列表: Vec<逻辑段>,
}

impl 地址空间 {
    fn 映射(&mut self, 逻辑段: 逻辑段) {
        for 虚拟页号 in 逻辑段.连续地址虚拟内存.虚拟页号范围() {
            let 物理页号;
            if 逻辑段.恒等映射 {
                物理页号 = 虚拟页号
            } else {
                物理页号 = 物理内存管理器::分配物理页并返回页号();
                self.frames.push(PPN::new(物理页号));
            }
            let flags;
            if 逻辑段.用户可见 {
                flags = PageTableEntryFlags::UXWR;
            } else {
                flags = PageTableEntryFlags::XWR;
            }
            self.page_table.map(VPN::new(虚拟页号), PPN::new(物理页号), flags);
        }
        self.逻辑段列表.push(逻辑段);
    }

    fn 新建空地址空间() -> Self {
        Self { 
            page_table: SV39PageTable::<TT>::new(),
            frames: Vec::new(),
            逻辑段列表: Vec::new(),
        }
    }

    pub fn 新建内核地址空间() -> Self {
        let mut 地址空间 = Self::新建空地址空间();

        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: stext as usize..etext as usize },
            恒等映射: true,
            用户可见: false,
        });
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: srodata as usize..erodata as usize },
            恒等映射: true,
            用户可见: false,
         });
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: sdata as usize..edata as usize },
            恒等映射: true,
            用户可见: false,
         });
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: sbss_with_stack as usize..ebss as usize },
            恒等映射: true,
            用户可见: false,
         });
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: ekernel as usize..可用物理内存结尾地址 },
            恒等映射: true,
            用户可见: false,
        });
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 0x100000..0x102000 },
            恒等映射: true,
            用户可见: false,
         }); // MMIO VIRT_TEST/RTC  in virt machine
        // 内核栈
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 内核栈栈顶() - 0x2000..内核栈栈顶() },
            恒等映射: false,
            用户可见: false,
        });
        地址空间
    }
    
    pub fn 新建应用地址空间(elf文件数据: &[u8]) -> (Self, usize, usize) {
        let mut 地址空间 = Self::新建空地址空间();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: __trap_entry as usize..__trap_end as usize },
            恒等映射: true,
            用户可见: false,
         });

        let elf文件 = Elf文件::解析(elf文件数据);
        let 程序段列表 = elf文件.程序段列表();
        for 程序段 in &程序段列表 {
            地址空间.映射(逻辑段 { 
                连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 程序段.虚拟地址范围() },
                恒等映射: false,
                用户可见: true,
             });
            地址空间.page_table.write(VA::new(程序段.虚拟地址范围().start), VA::new(程序段.虚拟地址范围().end), 程序段.数据);
        }

        let 最后一个程序段的虚拟地址范围 = 程序段列表.last().unwrap().虚拟地址范围();

        let 用户栈栈底 = 连续地址虚拟内存 { 虚拟地址范围: 最后一个程序段的虚拟地址范围 }.对齐到分页的结尾地址();
        let 用户栈栈顶 = 用户栈栈底 + 0x2000;
        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 用户栈栈底..用户栈栈顶 },
            恒等映射: false,
            用户可见: true,
         });

        地址空间.映射(逻辑段 { 
            连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 应用陷入上下文存放地址()..0xfffffffffffff000 },
            恒等映射: false,
            用户可见: false,
         });
        
        (
            地址空间,
            用户栈栈顶,
            elf文件.入口地址(),
        )
    }

    pub fn 复制地址空间(被复制的地址空间: &Self) -> Self {
        let mut 地址空间 = Self::新建空地址空间();
        for 逻辑段 in &被复制的地址空间.逻辑段列表 {
            let 虚拟地址范围 = 逻辑段.连续地址虚拟内存.虚拟地址范围.clone();
            地址空间.映射(逻辑段 {
                连续地址虚拟内存: 连续地址虚拟内存 { 虚拟地址范围: 虚拟地址范围.clone() },
                恒等映射: 逻辑段.恒等映射,
                用户可见: 逻辑段.用户可见
            });
            // TODO: 整理页表的完全复制，为何不能读完一部分数据再写入呢
            for vpn in 逻辑段.连续地址虚拟内存.虚拟页号范围() {
                let src_ppn = 被复制的地址空间.page_table.translate(VPN::new(vpn)).0;
                let dst_ppn = 地址空间.page_table.translate(VPN::new(vpn)).0;
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
    pub fn 陷入上下文(&self) -> &'static mut 陷入上下文 {
        let pa_ranges = self.page_table.translate_addr(VA::new(应用陷入上下文存放地址()), VA::new(0xfffffffffffff000));
        unsafe {
            &mut *(pa_ranges[0].0.0 as *mut 陷入上下文)
        }
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.page_table.root_ppn.0
    }

    pub fn 读取字节数组(&self, 虚拟地址范围: Range<usize>) -> Vec<u8> {
        self.page_table.read(VA::new(虚拟地址范围.start), VA::new(虚拟地址范围.end))
    }

    pub fn 写入字节数组(&self, 虚拟地址范围: Range<usize>, 数据: &[u8]) {
        self.page_table.write(VA::new(虚拟地址范围.start), VA::new(虚拟地址范围.end), &数据);
    }
}


impl Drop for 地址空间 {
    fn drop(&mut self) {
        for frame in &self.frames {
            物理内存管理器::回收物理帧(*frame);
        }
    }
}

lazy_static! {
    pub static ref 内核地址空间: 地址空间 = 地址空间::新建内核地址空间();
}
