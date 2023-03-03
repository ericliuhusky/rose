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
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    
    shutdown();
}
