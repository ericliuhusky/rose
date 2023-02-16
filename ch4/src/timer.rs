//! RISC-V timer-related functionality

use core::arch::asm;

const CLOCK_FREQ: usize = 12500000;
const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

/// read the `mtime` register
pub fn get_time() -> usize {
    let bits: usize;
    unsafe {
        asm!("csrrs {}, 0xc01, x0", out(reg) bits);
    }
    bits
}

/// get current time in milliseconds
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MSEC_PER_SEC)
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
