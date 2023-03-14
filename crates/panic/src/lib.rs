#![no_std]
#![feature(panic_info_message)]

extern crate print;

use core::panic::PanicInfo;
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
    #[cfg(feature = "user")]
    sys_call::exit(-1);
    #[cfg(not(feature = "user"))]
    sbi_call::shutdown()
}
