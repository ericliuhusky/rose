use core::arch::asm;

pub fn puts(s: &str) {
    for c in s.chars() {
        putchar(c);
    }
}

pub fn putchar(c: char) {
    put(c as usize)
}

#[inline(always)]
fn put(p: usize) {
    unsafe {
        asm!(
            "ecall",
            in("x10") p,
            in("x17") 1
        );
    }
}
