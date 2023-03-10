#[repr(C)]
pub struct Context {
    // x1~x31
    pub x: [usize; 32],
    pub sepc: usize,
}

impl Context {
    pub fn app_init(entry_address: usize, sp: usize) -> Self {
        let mut cx = Self {
            x: [0; 32],
            sepc: entry_address,
        };
        cx.x[2] = sp;
        cx
    }
}
