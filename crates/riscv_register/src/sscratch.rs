use core::arch::asm;

pub fn read() -> usize {
    let bits;
    unsafe {
        asm!("csrr {}, sscratch", out(reg) bits);
    }
    bits
}
