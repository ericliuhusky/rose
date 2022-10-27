#![no_std]

pub mod print;
pub mod yield_;
mod puts;
mod exit;
mod rust_no_std;


#[no_mangle]
#[link_section = ".text.entry"]
fn _start() {
    extern "C" {
        fn main() -> i32;
    }
    unsafe {
        exit::exit(main());
    }
}
