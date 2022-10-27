use core::arch::asm;

pub fn read() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrr {}, sstatus", out(reg) bits);
    }
    bits
}

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw sstatus, {}", in(reg) bits);
    }
}
