use crate::trap::陷入上下文;

const 用户栈栈顶: [usize; 3] = [
    0x80422000,
    0x80444000,
    0x80466000
];
const 内核栈栈顶: [usize; 3] = [
    0x80468000,
    0x8046a000,
    0x8046c000
];

fn 将上下文压入内核栈后的栈顶(上下文: 陷入上下文, 应用索引: usize) -> usize {
    let mut 栈顶 = 内核栈栈顶[应用索引];
    栈顶 -= core::mem::size_of::<陷入上下文>();
    let 上下文指针 = 栈顶 as *mut 陷入上下文;
    unsafe {
        *上下文指针 = 上下文;
    }
    栈顶
}

pub fn 读取应用数目() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

fn 读取应用数据(应用索引: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let 应用数目 = 读取应用数目();
    let 应用数目指针 = _num_app as usize as *const usize;
    unsafe {
        let 应用数据起始地址指针 = 应用数目指针.add(1);
        let 应用数据起始地址列表 = core::slice::from_raw_parts(应用数据起始地址指针, 应用数目 + 1);
        core::slice::from_raw_parts(
            应用数据起始地址列表[应用索引] as *const u8,
            应用数据起始地址列表[应用索引 + 1] - 应用数据起始地址列表[应用索引],
        )
    }
}

fn 加载应用到应用内存区(应用索引: usize) -> usize {
    unsafe {
        let 应用数据 = 读取应用数据(应用索引);
        let elf = elf_reader::ElfFile::read(应用数据);
            for p in elf.programs() {
                let start_va = p.virtual_address_range().start;
                let end_va = p.virtual_address_range().end;
                let dst = core::slice::from_raw_parts_mut(start_va as *mut u8, end_va - start_va);
                let src = p.data;
                dst.copy_from_slice(src);
            }
        elf.entry_address()
    }
}

pub fn 将应用初始上下文压入内核栈后的栈顶(应用索引: usize) -> usize {
    let ea = 加载应用到应用内存区(应用索引);
    将上下文压入内核栈后的栈顶(
        陷入上下文::应用初始上下文(
            ea,
            用户栈栈顶[应用索引]
        ),
        应用索引
    )
}
