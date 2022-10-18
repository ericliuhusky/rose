use crate::csr::{sstatus, sepc, sscratch};

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
    /// 保存控制和状态寄存器
    pub fn save_csr(&mut self) {
        self.sstatus = sstatus::read();
        self.sepc = sepc::read();
        // 保存用户栈
        self.x[2] = sscratch::read();
    }

    /// 恢复控制和状态寄存器
    pub fn restore_csr(&self) {
        sstatus::write(self.sstatus);
        sepc::write(self.sepc);
        // 恢复用户栈
        sscratch::write(self.x[2]);
    }
}
