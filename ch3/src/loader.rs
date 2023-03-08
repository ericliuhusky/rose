use crate::trap::陷入上下文;


static mut KERNEL_STACK_TOP: [usize; 3] = [0; 3];

pub fn init() {
    extern "C" {
        fn ekernel();
    }
    unsafe {
        KERNEL_STACK_TOP = [
            ekernel as usize + 0x2000,
            ekernel as usize + 2 * 0x2000,
            ekernel as usize + 3 * 0x2000
        ];
        for t in KERNEL_STACK_TOP {
            println!("{:#x}", t);
        }
    }
    
}

fn 将上下文压入内核栈后的栈顶(上下文: 陷入上下文, 应用索引: usize) -> usize {
    let mut 栈顶 = unsafe { KERNEL_STACK_TOP[应用索引] };
    栈顶 -= core::mem::size_of::<陷入上下文>();
    let 上下文指针 = 栈顶 as *mut 陷入上下文;
    unsafe {
        *上下文指针 = 上下文;
    }
    栈顶
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
    将上下文压入内核栈后的栈顶(
        陷入上下文::应用初始上下文(
            entry_address,
            user_stack_top
        ),
        应用索引
    )
}
