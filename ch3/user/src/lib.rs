#![no_std]

use 终止::终止;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> i32;
    }
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
}

mod 终止 {
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
