/// Trap Context
#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: usize,
    /// CSR sepc
    pub sepc: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// kernel stack
    pub kernel_sp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}

impl TrapContext {
    /// init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize
    ) -> Self {
        let mut cx = Self {
            x: [0; 32],
            sstatus: 0,
            sepc: entry, // entry point of app
            kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.x[2] = sp;
        cx // return initial Trap Context of app
    }
}
