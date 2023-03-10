use core::arch::asm;

#[inline(always)]
pub fn read() -> usize {
    let bits;
    unsafe {
        asm!("csrr {}, sscratch", out(reg) bits);
    }
    bits
}
