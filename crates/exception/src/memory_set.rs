use core::arch::asm;

#[inline(always)]
pub fn switch(satp: usize) {
    unsafe {
        riscv_register::satp::write(satp);
        asm!("sfence.vma");
    }
}

#[inline(always)]
pub fn switch_user() {
    extern "C" {
        fn USER_SATP();
    }
    unsafe {
        let satp = *(USER_SATP as *const usize);
        switch(satp);
    }
}

#[inline(always)]
pub fn switch_kernel() {
    extern "C" {
        fn KERNEL_SATP();
    }
    unsafe {
        let satp = *(KERNEL_SATP as *const usize);
        switch(satp);
    }
}
