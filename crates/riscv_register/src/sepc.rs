use core::arch::asm;

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw sepc, {}", in(reg) bits);
    }
}

pub fn read() -> usize {
    let bits;
    unsafe {
        asm!("csrr {}, sepc", out(reg) bits);
    }
    bits
}
