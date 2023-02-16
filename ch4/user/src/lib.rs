#![no_std]

pub mod print;
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

// 一个耗时程序，用以验证10ms之后会自动切换下一个任务
pub fn fibonacci(x: u32) -> u32 {
    if x == 0 { return 0 }
    if x == 1 { return 1 }
    fibonacci(x - 2) + fibonacci(x - 1)
}
