#[repr(C)]
pub struct Context {
    pub x: [usize; 32],
}

impl Context {
    /// 零初始化。
    pub const ZERO: Self = Self { x: [0; 32] };
}

macro_rules! registers {
    ($op:ident,$($i:expr),*) => {
        concat!(
            $(
                stringify!($op), " x", $i, ", ", $i, "*8(sp)\n",
            )*
        )
    };
}

macro_rules! save_registers {
    () => {
        registers!(
            sd, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30, 31
        )
    };
}

macro_rules! load_registers {
    () => {
        registers!(
            ld, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30, 31
        )
    };
}

pub static mut CONTEXT: Context = Context::ZERO;

#[naked]
pub extern "C" fn trap_entry() {
    unsafe {
        core::arch::asm!(
            ".align 2",
            // 换栈
            "csrrw sp, mscratch, sp",
            "la sp, {CONTEXT}",
            save_registers!(),
            "call {fast_handler}",
            // 加载上下文指针
            "la sp, {CONTEXT}",
            load_registers!(),
            "csrrw sp, mscratch, sp",
            "mret",
            CONTEXT = sym CONTEXT,
            fast_handler = sym crate::trap_handler,
            options(noreturn),
        )
    }
}
