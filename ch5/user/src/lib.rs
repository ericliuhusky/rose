#![no_std]
#![feature(alloc_error_handler)]

use 终止::终止;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> i32;
    }
    堆::初始化();
    let 终止代码 = unsafe { main() };
    终止(终止代码);
}

// 一个耗时程序，用以验证10ms之后会自动切换下一个任务
pub fn fibonacci(x: u32) -> u32 {
    if x == 0 { return 0 }
    if x == 1 { return 1 }
    fibonacci(x - 2) + fibonacci(x - 1)
}


pub mod 输出 {
    use core::arch::asm;
    use core::fmt::{self, Write};

    fn 输出(字符串: &str) {
        unsafe {
            asm!(
                "ecall",
                in("x10") 字符串.as_bytes().as_ptr() as usize,
                in("x11") 字符串.as_bytes().len(),
                in("x17") 1
            );
        }
    }

    struct 标准输出;
    impl Write for 标准输出 {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            输出(s);
            Ok(())
        }
    }
    pub fn 格式化输出(参数: fmt::Arguments) {
        标准输出.write_fmt(参数).unwrap();
    }

    #[macro_export]
    macro_rules! 格式化输出并换行 {
        ($fmt: literal $(, $($arg: tt)+)?) => {
            $crate::输出::格式化输出(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
        }
    }

    #[macro_export]
    macro_rules! println {
        ($fmt: literal $(, $($arg: tt)+)?) => {
            $crate::输出::格式化输出(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
        }
    }

    #[macro_export]
    macro_rules! print {
        ($fmt: literal $(, $($arg: tt)+)?) => {
            $crate::输出::格式化输出(format_args!($fmt $(, $($arg)+)?));
        }
    }
}

pub mod 终止 {
    use core::arch::asm;

    pub fn 终止(代码: i32) {
        unsafe {
            asm!(
                "ecall",
                in("x10") 代码 as usize,
                in("x17") 2
            );
        }
    }
}

mod rust裸机无标准库 {
    use core::panic::PanicInfo;

    // 需要提供崩溃处理
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}

mod 堆 {
    use buddy_system_allocator::LockedHeap;

    static mut 内核堆: [u8; 16384] = [0; 16384];

    #[global_allocator]
    static 堆管理器: LockedHeap = LockedHeap::empty();

    pub fn 初始化() {
        unsafe {
            堆管理器
                .lock()
                .init(内核堆.as_ptr() as usize, 内核堆.len());
        }
    }

    #[alloc_error_handler]
    pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
        panic!("Heap allocation error, layout = {:?}", layout);
    }
}


use core::arch::asm;

const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_EXIT: usize = 2;
const SYSCALL_YIELD: usize = 3;
const SYSCALL_GET_TIME: usize = 4;
const SYSCALL_GETPID: usize = 5;
const SYSCALL_FORK: usize = 6;
const SYSCALL_EXEC: usize = 7;
const SYSCALL_WAITPID: usize = 8;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_read(buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [buffer.as_mut_ptr() as usize, buffer.len(), 0],
    )
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, path.len(), 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(&mut c);
    c[0]
}
pub fn read(buf: &mut [u8]) -> isize {
    sys_read(buf)
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}
