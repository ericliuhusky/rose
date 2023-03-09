use crate::exception::Context;
use alloc::vec::Vec;
use sbi_call::shutdown;

#[no_mangle]
static mut KERNEL_STACK_TOP: usize = 0;
static mut CONTEXT_START_ADDRS: Vec<usize> = Vec::new();
#[no_mangle]
static mut CONTEXT_START_ADDR: usize = 0;
static mut APP_START_ADDR: usize = 0;

struct 任务 {
    状态: 任务状态,
    i: usize
}

#[derive(PartialEq)]
enum 任务状态 {
    就绪,
    运行,
    终止,
}

pub struct 任务管理器 {
    任务数目: usize,
    任务列表: Vec<任务>,
    当前任务索引: isize,
}

impl 任务管理器 {
    pub fn 初始化() {
        extern "C" {
            fn ekernel();
        }
        let n = loader::read_app_num();
        unsafe {
            KERNEL_STACK_TOP = ekernel as usize + 0x2000;
            for i in 0..n {
                CONTEXT_START_ADDRS.push(KERNEL_STACK_TOP +  i * core::mem::size_of::<Context>());
            }
            APP_START_ADDR = CONTEXT_START_ADDRS[n-1] + core::mem::size_of::<Context>();
        }

        let 任务数目 = loader::read_app_num();
        let mut 任务列表 = Vec::new();
        for i in 0..任务数目 {
            let (entry_address, user_stack_top) = 加载应用到应用内存区(i);
            assert!(entry_address > unsafe { APP_START_ADDR });
            unsafe {
                let cx_ptr = CONTEXT_START_ADDRS[i] as *mut Context;
                *cx_ptr = Context::app_init(
                    entry_address,
                    user_stack_top
                );
            }
            任务列表.push(任务 {
                i,
                状态: 任务状态::就绪
            })
        }
        unsafe {
            任务管理器 = 任务管理器 {
                任务数目,
                任务列表,
                当前任务索引: -1
            };
        }
    }

    fn 当前任务(&mut self) -> &mut 任务 {
        &mut self.任务列表[self.当前任务索引 as usize]
    }

    pub fn 暂停并运行下一个任务() {
        unsafe {
            任务管理器.当前任务().状态 = 任务状态::就绪;
            Self::运行下一个任务();
        }
    }

    pub fn 终止并运行下一个任务() {
        unsafe {
            任务管理器.当前任务().状态 = 任务状态::终止;
            Self::运行下一个任务();
        }
    }

    fn 查找下一个就绪任务(&mut self) -> Option<&mut 任务> {
        let 下一个任务索引 = (self.当前任务索引 + 1) as usize;
        let 下一个就绪任务索引 = (下一个任务索引..下一个任务索引 + self.任务数目)
            .map(|任务索引| 任务索引 % self.任务数目)
            .find(|任务索引| self.任务列表[*任务索引].状态 == 任务状态::就绪);
        if let Some(下一个就绪任务索引) = 下一个就绪任务索引 {
            self.当前任务索引 = 下一个就绪任务索引 as isize;
            Some(&mut self.任务列表[下一个就绪任务索引])
        } else {
            None
        }
    }

    pub fn 运行下一个任务() {
        unsafe {
            if let Some(下一个任务) = 任务管理器.查找下一个就绪任务() {
                下一个任务.状态 = 任务状态::运行;
                CONTEXT_START_ADDR = CONTEXT_START_ADDRS[下一个任务.i];
                extern "C" {
                    fn __restore();
                }
                __restore();
            } else {
                println!("[Kernel] All applications completed!");
                shutdown();
            }
        }
    }
}

static mut 任务管理器: 任务管理器 = 任务管理器 {
    任务数目: 0,
    任务列表: Vec::new(),
    当前任务索引: 0
};


fn 加载应用到应用内存区(应用索引: usize) -> (usize, usize) {
    unsafe {
        let 应用数据 = loader::read_app_data(应用索引);
        let elf = elf_reader::ElfFile::read(应用数据);
        println!("{:x}", elf.entry_address());
        for p in elf.programs() {
            let start_va = p.virtual_address_range().start;
            let end_va = p.virtual_address_range().end;
            println!("{:x},{:x}", start_va, end_va);
            if start_va < 0x80200000 {
                continue;
            }
            let dst = core::slice::from_raw_parts_mut(start_va as *mut u8, end_va - start_va);
            let src = p.data;
            let len = dst.len().min(src.len());
            for j in 0..len {
                dst[j] = src[j];
            }
        }
        let last_p_va_end = elf.programs().last().unwrap().virtual_address_range().end;
        let user_stack_top = last_p_va_end +0x2000;
        (elf.entry_address(), user_stack_top)
    }
}
