#![no_std]

#[macro_use]
mod macros;
pub mod cell;
pub mod io;
pub mod panic;

pub trait CoreExt {
    fn putchar(&self, c: char);
    fn exit(&self) -> !;
}

static_var! {
    CORE_EXT: Option<&'static dyn CoreExt> = None;
}

pub fn init(core_ext_impl: &'static dyn CoreExt) {
    CORE_EXT::set(Some(core_ext_impl));
}

fn self_impl() -> &'static dyn CoreExt {
    CORE_EXT.unwrap()
}

pub struct UInt(pub usize);

impl UInt {
    /// 以x为对齐标准，向下对齐
    ///
    /// 仅当x是2的幂时有效
    pub fn align_to_lower(&self, x: usize) -> usize {
        self.0 & !(x - 1)
    }

    /// 以x为对齐标准，向下对齐
    ///
    /// 仅当x是2的幂时有效
    pub fn align_to_upper(&self, x: usize) -> usize {
        (self.0 + x - 1) & !(x - 1)
    }
}
