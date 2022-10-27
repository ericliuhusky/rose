use core::arch::asm;

pub fn puts(s: &str) {
    unsafe {
        asm!(
            "ecall",
            in("x10") s.as_bytes().as_ptr() as usize,
            in("x11") s.as_bytes().len(),
            in("x17") 1
        );
    }
}
