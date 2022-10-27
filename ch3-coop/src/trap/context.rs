/// Trap Context
#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: usize,
    /// CSR sepc
    pub sepc: usize,
}

impl TrapContext {
    /// init app context
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut cx = Self {
            x: [0; 32],
            sstatus: 0,
            sepc: entry, // entry point of app
        };
        cx.x[2] = sp;
        cx // return initial Trap Context of app
    }
}
