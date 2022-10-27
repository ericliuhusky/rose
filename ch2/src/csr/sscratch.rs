use core::arch::asm;

pub fn read() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrr {}, sscratch", out(reg) bits);
    }
    bits
}

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw sscratch, {}", in(reg) bits);
    }
}
