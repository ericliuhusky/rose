#![no_std]
#![no_main]

extern crate user;
use user::格式化输出并换行;
use user::fibonacci;

#[no_mangle]
fn main() -> i32 {
    for i in 1..=30 {
        let r = fibonacci(i);
        格式化输出并换行!("B [{}/{}] {}", i, 30, r);
    }
    0
}
