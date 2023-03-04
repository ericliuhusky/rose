use core::arch::asm;

pub fn read() -> Exception {
    let bits: usize;
    unsafe {
        asm!("csrr {}, scause", out(reg) bits);
    }
    if (bits >> 63) & 1 == 1 {
        Exception::Interrupt(Interrupt::from(bits & !(1 << 63)))
    } else {
        Exception::from(bits)
    }
}

pub enum Exception {
    UserEnvCall,
    StoreFault,
    StorePageFault,
    IllegalInstruction,
    Interrupt(Interrupt),
    Unknown
}

pub enum Interrupt {
    Timer,
    Unknown
}

impl Exception {
    fn from(n: usize) -> Self {
        match n {
            2 => Exception::IllegalInstruction,
            7 => Exception::StoreFault,
            15 => Exception::StorePageFault,
            8 => Exception::UserEnvCall,
            _ => Exception::Unknown
        }
    }
}

impl Interrupt {
    fn from(n: usize) -> Self {
        match n {
            5 => Interrupt::Timer,
            _ => Interrupt::Unknown
        }
    }
}
