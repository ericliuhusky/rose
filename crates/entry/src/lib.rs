#![no_std]
#![feature(linkage)]

use core::arch::asm;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    unsafe {
        // TODO: KERNEL_STACK_TOP
        asm!("li sp, 0x87800000");
        asm!("call main")
    }
}
