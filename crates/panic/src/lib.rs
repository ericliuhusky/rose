#![no_std]
#![feature(panic_info_message)]

extern crate sbi_call;
extern crate print;

use core::panic::PanicInfo;
use sbi_call::shutdown;
use print::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
