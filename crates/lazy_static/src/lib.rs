#![no_std]

pub mod once;

#[macro_export]
macro_rules! lazy_static {
    (pub static $N:ident : $T:ty = $e:expr;) => {
        struct $N {}
        static $N: $N = $N {};
        impl core::ops::Deref for $N {
            type Target = $T;
            fn deref(&self) -> &$T {
                static mut LAZY: $crate::once::Once<$T> = $crate::once::Once::new();
                unsafe { LAZY.once(|| $e) }
            }
        }
    }
}
