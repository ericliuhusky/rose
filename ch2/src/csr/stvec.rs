use core::arch::asm;

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw stvec, {}", in(reg) bits);
    }
}
