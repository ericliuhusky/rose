use core::arch::asm;

pub fn write(bits: usize) {
    unsafe {
        asm!("csrw satp, {}", in(reg) bits);
    }
}
