#![no_std]
#![no_main]

#[no_mangle]
fn _start() {
    puts("Hello, world!\n");
    shutdown();
}

fn puts(s: &str) {
    for c in s.chars() {
        putchar(c as usize);
    }
}

use core::arch::asm;

fn putchar(c: usize) {
    unsafe {
        asm!(
            "ecall",
            in("x10") c,
            in("x17") 1
        );
    }
}

fn shutdown() {
    unsafe {
        asm!(
            "ecall",
            in("x17") 8
        );
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    panic!()
}
