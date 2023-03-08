use core::arch::asm;

/// trap vector base address register
pub fn write(bits: usize) {
    unsafe {
        asm!("csrw stvec, {}", in(reg) bits);
    }
}
