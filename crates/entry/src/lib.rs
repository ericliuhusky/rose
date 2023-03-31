#![no_std]
#![feature(linkage)]

use core::arch::asm;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    let top = BOOT_STACK.as_ptr() as usize + BOOT_STACK.len();
    unsafe {
        asm!("mv sp, {}", in(reg) top);
        asm!("call main")
    }
}

#[link_section = ".data"]
static BOOT_STACK: [u8; 0x10000] = [0; 0x10000];
