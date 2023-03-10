use core::arch::asm;

#[inline(always)]
pub fn enable_timer_interrupt() {
    unsafe {
        asm!("csrw sie, {}", in(reg) 1 << 5);
    }
}
