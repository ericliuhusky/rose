#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

#[no_mangle]
fn main() {
    for i in 1..=30 {
        let r = fibonacci(i);
        println!("A [{}/{}] {}", i, 30, r);
    }
}

// 一个耗时程序，用以验证10ms之后会自动切换下一个任务
pub fn fibonacci(x: u32) -> u32 {
    if x == 0 { return 0 }
    if x == 1 { return 1 }
    fibonacci(x - 2) + fibonacci(x - 1)
}
