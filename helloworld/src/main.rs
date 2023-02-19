#![no_std]
#![no_main]

use 输出::输出;
use 终止::终止;

#[no_mangle]
fn _start() {
    输出("Hello, world!\n");
    终止();
}

mod 输出 {
    use core::arch::asm;

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
