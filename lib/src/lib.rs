#![no_std]

use print::println;
use sys_call::exit;
use panic;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main();
    }
    unsafe { main(); }
    exit(0);
}
