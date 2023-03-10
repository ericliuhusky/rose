use core::arch::asm;

#[inline(always)]
pub fn read() -> usize {
    let bits;
    unsafe {
        asm!("csrr {}, time", out(reg) bits);
    }
    bits
}
