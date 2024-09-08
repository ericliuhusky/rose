#[repr(C)]
pub struct Context {
    pub x: [usize; 32],
}

impl Context {
    /// 零初始化。
    pub const ZERO: Self = Self { x: [0; 32] };
}

pub static mut CONTEXT: Context = Context::ZERO;

#[naked]
pub unsafe extern "C" fn trap_entry() {
    core::arch::asm!(
        ".align 2",
        // 换栈
        write_a0_to_scratch!(m),
        "la a0, {CONTEXT}",
        store_registers_to_a0!(),
        read_scratch_and_store_to_a0!(m),
        "call {fast_handler}",
        // 加载上下文指针
        "la a0, {CONTEXT}",
        load_registers_from_a0!(),
        "ld a0, 10*8(a0)",
        "mret",
        CONTEXT = sym CONTEXT,
        fast_handler = sym crate::trap_handler,
        options(noreturn),
    )
}
