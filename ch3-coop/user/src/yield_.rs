use core::arch::asm;

pub fn yield_() {
    unsafe {
        asm!(
            "ecall",
            in("x17") 3
        );
    }
}
