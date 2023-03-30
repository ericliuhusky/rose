use core::arch::asm;

#[inline(always)]
pub fn switch(satp: usize) {
    unsafe {
        riscv_register::satp::write(satp);
        asm!("sfence.vma");
    }
}

#[inline(always)]
pub fn switch_user(satp: usize) {
    switch(satp);
}

#[link_section = ".text.trampoline"]
static mut KERNEL_SATP: usize = 0;

pub fn set_kernel_satp(satp: usize) {
    unsafe {
        KERNEL_SATP = satp;
    }
}

#[inline(always)]
pub fn switch_kernel() {
    switch(unsafe { KERNEL_SATP });
}
