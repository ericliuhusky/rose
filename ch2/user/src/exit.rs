use core::arch::asm;

pub fn exit(code: i32) {
    unsafe {
        asm!(
            "ecall",
            in("x10") code as usize,
            in("x17") 2
        );
    }
}
