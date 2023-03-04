use core::arch::asm;

pub fn read() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrr {}, time", out(reg) bits);
    }
    bits
}
