use crate::trap::陷入上下文;
use sbi_call::shutdown;
const 应用内存区起始地址: usize = 0x80400000;
const 应用内存区大小限制: usize = 0x20000;
const 用户栈栈顶: usize = 0x80422000;
const 内核栈栈顶: usize = 0x80424000;

fn 将上下文压入内核栈后的栈顶(上下文: 陷入上下文) -> usize {
    let mut 栈顶 = 内核栈栈顶;
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
    fn 加载应用到应用内存区(&self, 应用索引: usize) {
        if 应用索引 >= self.应用数目 {
            println!("[kernel] All applications completed!");
            shutdown();
        }
        println!("[kernel] Loading app_{}", 应用索引);
        unsafe {
            // 清空应用内存区
            core::slice::from_raw_parts_mut(应用内存区起始地址 as *mut u8, 应用内存区大小限制).fill(0);

            let 应用数据 = 读取应用数据(应用索引);
            let 应用占用的内存 = core::slice::from_raw_parts_mut(应用内存区起始地址 as *mut u8, 应用数据.len());
            应用占用的内存.copy_from_slice(应用数据);
        }
    }

    pub fn 初始化() {
        unsafe {
            let 应用数目 = 读取应用数目();
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
            应用管理器.加载应用到应用内存区(当前应用索引);
            应用管理器.当前应用索引 += 1;

            extern "C" {
                fn __restore(cx_addr: usize);
            }
            __restore(
                将上下文压入内核栈后的栈顶(
                    陷入上下文::应用初始上下文(
                        应用内存区起始地址,
                        用户栈栈顶
                    )
                )
            );
        }
    }
}


static mut 应用管理器: 应用管理器 = 应用管理器 {应用数目:0, 当前应用索引:0};

fn 读取应用数目() -> usize {
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
