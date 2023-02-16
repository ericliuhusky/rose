//! RISC-V timer-related functionality

use core::arch::asm;

const CLOCK_FREQ: usize = 12500000;
const TICKS_PER_SEC: usize = 100;

/// read the `mtime` register
pub fn get_time() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrrs {}, 0xc01, x0", out(reg) bits);
    }
    bits
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn set_timer(timer: usize) {
    unsafe {
        asm!(
            "ecall",
            in("x10") timer,
            in("x17") 0,
        );
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        asm!("csrrs x0, 0x104, {}", in(reg) 1 << 5);
    }
}
