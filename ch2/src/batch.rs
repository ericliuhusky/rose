use crate::trap::陷入上下文;
use sbi_call::shutdown;

extern "C" {
    fn ekernel();
}

const 用户栈栈顶: usize = 0x80422000;
static mut KENRL_STACK_TOP: usize = 0;

fn 将上下文压入内核栈后的栈顶(上下文: 陷入上下文) -> usize {
    let mut 栈顶 = unsafe { KENRL_STACK_TOP };
    栈顶 -= core::mem::size_of::<陷入上下文>();
    let 上下文指针 = 栈顶 as *mut 陷入上下文;
    unsafe {
        *上下文指针 = 上下文;
    }
    栈顶
}

pub struct 应用管理器 {
    应用数目: usize,
    当前应用索引: usize
}

impl 应用管理器 {
    fn 加载应用到应用内存区(&self, 应用索引: usize) -> usize {
        if 应用索引 >= self.应用数目 {
            println!("[kernel] All applications completed!");
            shutdown();
        }
        println!("[kernel] Loading app_{}", 应用索引);
        unsafe {
            // 清空
            core::slice::from_raw_parts_mut(0x80400000 as *mut u8, 0x20000).fill(0);
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
            elf.entry_address()
        }
    }

    pub fn 初始化() {
        unsafe {
            KENRL_STACK_TOP = ekernel as usize + 0x2000;
            let 应用数目 = loader::read_app_num();
            应用管理器 = Self {
                应用数目,
                当前应用索引: 0,
            };
    
            println!("[kernel] num_app = {}", 应用数目);
        }
    }

    pub fn 运行下一个应用() {
        unsafe {
            let 当前应用索引 = 应用管理器.当前应用索引;
            let ea = 应用管理器.加载应用到应用内存区(当前应用索引);
            应用管理器.当前应用索引 += 1;

            extern "C" {
                fn __restore(cx_addr: usize);
            }
            __restore(
                将上下文压入内核栈后的栈顶(
                    陷入上下文::应用初始上下文(
                        ea,
                        用户栈栈顶
                    )
                )
            );
        }
    }
}


static mut 应用管理器: 应用管理器 = 应用管理器 {应用数目:0, 当前应用索引:0};
