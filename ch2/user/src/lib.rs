#![no_std]

use print::println;
use panic;

#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> isize;
    }
    let exit_code = unsafe { main() };
    sys_call::exit(exit_code);
}
