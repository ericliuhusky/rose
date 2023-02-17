#![no_std]
#![no_main]

extern crate user;
use user::格式化输出并换行;
use core::arch::asm;

#[no_mangle]
fn main() -> i32 {
    格式化输出并换行!("Try to execute privileged instruction in U Mode");
    格式化输出并换行!("Kernel should kill this application!");
    unsafe {
        asm!("sret");
    }
    0
}
