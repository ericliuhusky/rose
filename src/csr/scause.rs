pub struct Scause {
    pub bits: usize
}

impl Scause {
    pub fn cause(&self) -> Exception {
        Exception::from(self.bits)
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
