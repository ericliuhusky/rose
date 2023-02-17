#![no_std]

use 退出::退出;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> i32;
    }
    let 退出代码 = unsafe { main() };
    退出(退出代码);
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

mod 退出 {
    use core::arch::asm;

    pub fn 退出(代码: i32) {
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
