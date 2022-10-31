pub struct Scause {
    pub bits: usize
}

impl Scause {
    pub fn cause(&self) -> Trap {
        if self.is_interrupt() {
            Trap::Interrupt(Interrupt::from(self.code()))
        } else {
            Trap::Exception(Exception::from(self.code()))
        }
    }

    fn is_interrupt(&self) -> bool {
        (self.bits & (1 << (core::mem::size_of::<usize>() * 8 - 1))) != 0
    }

    fn code(&self) -> usize {
        let bit = 1 << (core::mem::size_of::<usize>() * 8 - 1);
        self.bits & !bit
    }
}

pub fn read() -> Scause {
    let bits: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) bits);
    }
    Scause { bits }
}

pub enum Exception {
    UserEnvCall,
    StoreFault,
    StorePageFault,
    IllegalInstruction,
    Unknown
}

pub enum Interrupt {
    SupervisorTimer,
    Unknown
}

pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
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
    pub fn from(n: usize) -> Self {
        match n {
            5 => Interrupt::SupervisorTimer,
            _ => Interrupt::Unknown
        }
    }
}
