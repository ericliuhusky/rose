#![no_std]
#![no_main]

mod syscall;
mod trap;
mod batch;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));
global_asm!(include_str!("link_app.s"));

#[no_mangle]
fn rust_main() {
    格式化输出并换行!("[kernel] Hello, world!");
    trap::初始化();
    batch::应用管理器::初始化();
    batch::应用管理器::运行下一个应用();
}

mod 输出 {
    use core::arch::asm;
    use core::fmt::{self, Write};

    pub fn 输出(字符串: &str) {
        for 字符 in 字符串.chars() {
            unsafe {
                asm!(
                    "ecall",
                    in("x10") 字符 as usize,
                    in("x17") 1
                );
            }
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

    pub fn 终止() {
        unsafe {
            asm!(
                "sw {0}, 0({1})",
                in(reg)0x5555, in(reg)0x100000
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
