use core::arch::asm;

/// trap vector base address register
#[inline(always)]
pub fn write(bits: usize) {
    unsafe {
        asm!("csrw stvec, {}", in(reg) bits);
    }
}
