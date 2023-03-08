use alloc::vec::Vec;
use crate::trap::陷入上下文;


static mut KERNEL_STACK_TOP: Vec<usize> = Vec::new();

pub fn init() {
    extern "C" {
        fn ekernel();
    }
    let n = loader::read_app_num();
    unsafe {
        for i in 0..n {
            KERNEL_STACK_TOP.push(ekernel as usize + (i + 1) * 0x2000);
        }
    }
}



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

pub fn 将应用初始上下文压入内核栈后的栈顶(应用索引: usize) -> usize {
    let (entry_address, user_stack_top) = 加载应用到应用内存区(应用索引);
    let cx_addr = unsafe { KERNEL_STACK_TOP[应用索引] } - core::mem::size_of::<陷入上下文>();
    let cx_ptr = cx_addr as *mut 陷入上下文;
    unsafe {
        *cx_ptr = 陷入上下文::应用初始上下文(
            entry_address,
            user_stack_top
        );
    }
    cx_addr
}
