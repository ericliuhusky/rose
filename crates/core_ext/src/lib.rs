#![no_std]

extern crate alloc;

#[macro_use]
mod macros;
pub mod io;
pub mod panic;

use alloc::boxed::Box;

pub trait CoreExt {
    fn putchar(&self, c: char);
    fn exit(&self) -> !;
}

pub static mut CORE_EXT: Option<Box<dyn CoreExt>> = None;

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
