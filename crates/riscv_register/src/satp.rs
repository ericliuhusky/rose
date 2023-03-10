use core::arch::asm;

#[inline(always)]
pub fn write(bits: usize) {
    unsafe {
        asm!("csrw satp, {}", in(reg) bits);
    }
}
