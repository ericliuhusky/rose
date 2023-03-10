use core::arch::asm;

#[inline(always)]
pub fn write(bits: usize) {
    unsafe {
        asm!("csrw sepc, {}", in(reg) bits);
    }
}

#[inline(always)]
pub fn read() -> usize {
    let bits;
    unsafe {
        asm!("csrr {}, sepc", out(reg) bits);
    }
    bits
}
