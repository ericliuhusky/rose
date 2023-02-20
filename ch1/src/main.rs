#![no_std]
#![no_main]

use core::arch::global_asm;
use 终止::终止;

global_asm!(include_str!("entry.s"));

#[no_mangle]
fn rust_main() {
    extern "C" {
        fn stext(); // text段起始地址
        fn etext(); // text段结尾地址
        fn srodata(); // 只读数据段起始地址
        fn erodata(); // 只读数据段结尾地址
        fn sdata(); // 数据段起始地址
        fn edata(); // 数据段结尾地址
        fn sbss(); // bss段起始地址
        fn ebss(); // bss段结尾地址
        fn boot_stack(); // 栈底
        fn boot_stack_top(); // 栈顶
    }
    
    格式化输出并换行!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    格式化输出并换行!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    格式化输出并换行!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    格式化输出并换行!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    格式化输出并换行!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    
    终止();
}

mod 输出 {
    use core::arch::asm;
    use core::fmt::{self, Write};

    fn 输出(字符串: &str) {
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
