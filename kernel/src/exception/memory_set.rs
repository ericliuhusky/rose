use core::arch::asm;

#[inline(always)]
pub fn switch(satp: usize) {
    unsafe {
        riscv::register::satp::write(satp);
        asm!("sfence.vma");
    }
}

#[link_section = ".text.trampoline"]
pub static mut KERNEL_SATP: usize = 0;

pub fn set_kernel_satp(satp: usize) {
    unsafe {
        KERNEL_SATP = satp;
    }
}

#[inline(always)]
pub fn switch_kernel() {
    switch(unsafe { KERNEL_SATP });
}
