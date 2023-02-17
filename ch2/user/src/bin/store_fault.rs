#![no_std]
#![no_main]

extern crate user;
use user::格式化输出并换行;

#[no_mangle]
fn main() -> i32 {
    格式化输出并换行!("Into Test store_fault, we will insert an invalid store operation...");
    格式化输出并换行!("Kernel should kill this application!");
    unsafe {
        let 空指针 = core::ptr::null_mut::<u8>();
        空指针.write_volatile(0);
    }
    0
}
