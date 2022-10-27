use core::arch::asm;

pub fn exit() {
    unsafe {
        asm!(
            "sw {0}, 0({1})",
            in(reg)0x5555, in(reg)0x100000
        );
    }
}
