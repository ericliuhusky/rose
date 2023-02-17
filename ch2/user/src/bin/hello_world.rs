#![no_std]
#![no_main]

extern crate user;
use user::格式化输出并换行;

#[no_mangle]
fn main() -> i32 {
    格式化输出并换行!("Hello, world!");
    0
}
