use crate::trap::陷入上下文;
use crate::格式化输出并换行;
use crate::退出::退出;

const 用户栈大小: usize = 4096 * 2;
const 内核栈大小: usize = 4096 * 2;
const 应用程序内存区起始地址: usize = 0x80400000;
const 应用程序内存区大小限制: usize = 0x20000;

struct 内核栈 {
    数据: [u8; 内核栈大小],
}

struct 用户栈 {
    数据: [u8; 用户栈大小],
}

static 内核栈: 内核栈 = 内核栈 { 数据: [0; 内核栈大小] };
static 用户栈: 用户栈 = 用户栈 { 数据: [0; 用户栈大小] };

impl 内核栈 {
    fn 将上下文压入内核栈后的栈顶(&self, 上下文: 陷入上下文) -> usize {
        let mut 栈顶 = self.数据.as_ptr() as usize + 内核栈大小;
        栈顶 -= core::mem::size_of::<陷入上下文>();
        let 上下文指针 = 栈顶 as *mut 陷入上下文;
        unsafe {
            *上下文指针 = 上下文;
        }
        栈顶
    }
}

impl 用户栈 {
    fn 栈顶(&self) -> usize {
        self.数据.as_ptr() as usize + 用户栈大小
    }
}

pub struct 批处理系统 {
    应用程序数目: usize,
    当前应用程序索引: usize
}

impl 批处理系统 {
    fn 加载应用程序到应用程序内存区(&self, 应用程序索引: usize) {
        if 应用程序索引 >= self.应用程序数目 {
            格式化输出并换行!("[kernel] All applications completed!");
            退出();
        }
        格式化输出并换行!("[kernel] Loading app_{}", 应用程序索引);
        unsafe {
            // 清空应用程序内存区
            core::slice::from_raw_parts_mut(应用程序内存区起始地址 as *mut u8, 应用程序内存区大小限制).fill(0);

            let 应用程序数据 = 读取应用程序数据(应用程序索引);
            let 应用程序占用的内存 = core::slice::from_raw_parts_mut(应用程序内存区起始地址 as *mut u8, 应用程序数据.len());
            应用程序占用的内存.copy_from_slice(应用程序数据);
        }
    }

    pub fn 初始化() {
        unsafe {
            let 应用程序数目 = 读取应用程序数目();
            批处理系统 = Self {
                应用程序数目,
                当前应用程序索引: 0,
            };
    
            格式化输出并换行!("[kernel] num_app = {}", 应用程序数目);
        }
    }

    pub fn 运行下一个应用程序() {
        unsafe {
            let 当前应用程序索引 = 批处理系统.当前应用程序索引;
            批处理系统.加载应用程序到应用程序内存区(当前应用程序索引);
            批处理系统.当前应用程序索引 += 1;

            extern "C" {
                fn __restore(cx_addr: usize);
            }
            __restore(
                内核栈.将上下文压入内核栈后的栈顶(
                    陷入上下文::应用程序初始上下文(
                        应用程序内存区起始地址,
                        用户栈.栈顶()
                    )
                )
            );
        }
    }
}


static mut 批处理系统: 批处理系统 = 批处理系统 {应用程序数目:0, 当前应用程序索引:0};

fn 读取应用程序数目() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

fn 读取应用程序数据(应用程序索引: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let 应用程序数目 = 读取应用程序数目();
    let 应用程序数目指针 = _num_app as usize as *const usize;
    unsafe {
        let 应用程序数据起始地址指针 = 应用程序数目指针.add(1);
        let 应用程序数据起始地址列表 = core::slice::from_raw_parts(应用程序数据起始地址指针, 应用程序数目 + 1);
        core::slice::from_raw_parts(
            应用程序数据起始地址列表[应用程序索引] as *const u8,
            应用程序数据起始地址列表[应用程序索引 + 1] - 应用程序数据起始地址列表[应用程序索引],
        )
    }
}
