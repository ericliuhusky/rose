#![no_std]
#![no_main]

use core::arch::global_asm;
use sbi_call::shutdown;
use print::println;
use panic;

global_asm!(include_str!("entry.s"));

#[no_mangle]
fn rust_main() {
    extern "C" {
        fn boot_stack(); // 栈底
        fn boot_stack_top(); // 栈顶
    }
    println!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!("Hello, world!");
    shutdown();
}
