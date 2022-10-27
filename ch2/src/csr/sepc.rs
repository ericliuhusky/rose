use core::arch::asm;

pub fn read() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrr {}, sepc", out(reg) bits);
    }
    bits
}

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw sepc, {}", in(reg) bits);
    }
}
