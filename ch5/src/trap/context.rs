#[repr(C)]
pub struct Context {
    // x1~x31
    pub x: [usize; 32],
    pub sepc: usize,
    pub kernel_satp: usize,
}

impl Context {
    pub fn app_init(entry_address: usize, sp: usize, kernel_satp: usize) -> Self {
        let mut cx = Self {
            x: [0; 32],
            sepc: entry_address,
            kernel_satp
        };
        cx.x[2] = sp;
        cx
    }
}
