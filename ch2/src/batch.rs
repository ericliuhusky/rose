use crate::trap::陷入上下文;
use sbi_call::shutdown;

static mut KENRL_STACK_TOP: usize = 0;

pub struct 应用管理器 {
    应用数目: usize,
    当前应用索引: usize,
}

impl 应用管理器 {
    fn 加载应用到应用内存区(&self, 应用索引: usize) -> (usize, usize) {
        if 应用索引 >= self.应用数目 {
            println!("[kernel] All applications completed!");
            shutdown();
        }
        unsafe {
            // 清空
            core::slice::from_raw_parts_mut(0x80500000 as *mut u8, 0x20000).fill(0);
            let 应用数据 = loader::read_app_data(应用索引);
            let elf = elf_reader::ElfFile::read(应用数据);
            assert!(elf.entry_address() > KENRL_STACK_TOP);
            for p in elf.programs() {
                let start_va = p.virtual_address_range().start;
                let end_va = p.virtual_address_range().end;
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
            let last_p_va_range = elf.programs().last().unwrap().virtual_address_range();
            let user_stack_top = last_p_va_range.end + 0x2000;
            (user_stack_top, elf.entry_address())
        }
    }

    pub fn 初始化() {
        extern "C" {
            fn ekernel();
        }
        unsafe {
            KENRL_STACK_TOP = ekernel as usize + 0x2000;
            let 应用数目 = loader::read_app_num();
            应用管理器 = Self {
                应用数目,
                当前应用索引: 0,
            };
        }
    }

    pub fn 运行下一个应用() {
        unsafe {
            let 当前应用索引 = 应用管理器.当前应用索引;
            let (user_stack_top, entry) = 应用管理器.加载应用到应用内存区(当前应用索引);
            应用管理器.当前应用索引 += 1;

            let cx_addr = KENRL_STACK_TOP - core::mem::size_of::<陷入上下文>();
            let cx_ptr = cx_addr as *mut 陷入上下文;
            *cx_ptr = 陷入上下文::应用初始上下文(entry, user_stack_top);
            extern "C" {
                fn __restore(cx_ptr: *mut 陷入上下文);
            }
            __restore(cx_ptr);
        }
    }
}

static mut 应用管理器: 应用管理器 = 应用管理器 {
    应用数目: 0,
    当前应用索引: 0,
};
