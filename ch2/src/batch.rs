use crate::exception::Context;
use sbi_call::shutdown;
use crate::segment::{CONTEXT_START_ADDR, APP_START_ADDR, APP_END_ADDR};


pub struct 应用管理器 {
    应用数目: usize,
    当前应用索引: usize,
}

impl 应用管理器 {
    fn 加载应用到应用内存区(&self, i: usize) -> (usize, usize) {
        if i >= self.应用数目 {
            println!("[kernel] All applications completed!");
            shutdown();
        }
        unsafe {
            let app_data = loader::read_app_data(i);
            let elf = elf_reader::ElfFile::read(app_data);
            let entry_address = elf.entry_address();
            assert!(entry_address > APP_START_ADDR);
            let last_p_va_end = elf.programs().last().unwrap().virtual_address_range().end;
            let user_stack_top = last_p_va_end + 0x2000;
            APP_END_ADDR = user_stack_top;
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
            (entry_address, user_stack_top)
        }
    }

    pub fn 初始化() {
        unsafe {
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
            let (entry_address, user_stack_top) = 应用管理器.加载应用到应用内存区(当前应用索引);
            应用管理器.当前应用索引 += 1;

            let cx_ptr = CONTEXT_START_ADDR as *mut Context;
            *cx_ptr = Context::app_init(entry_address, user_stack_top);
            extern "C" {
                fn __restore();
            }
            __restore();
        }
    }

    pub fn recycle() {
        unsafe {
            for addr in APP_START_ADDR..APP_END_ADDR {
                *(addr as *mut u8) = 0;
            }
        }
    }
}

static mut 应用管理器: 应用管理器 = 应用管理器 {
    应用数目: 0,
    当前应用索引: 0,
};
