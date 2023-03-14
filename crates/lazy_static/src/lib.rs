#![no_std]

pub mod once;

#[macro_export(local_inner_macros)]
macro_rules! lazy_static {
    (pub static ref $N:ident : $T:ty = $e:expr;) => {
        __lazy_static!((pub) static $N : $T = $e;);
    };
    (static ref $N:ident : $T:ty = $e:expr;) => {
        __lazy_static!(() static $N : $T = $e;);
    }
}

#[macro_export(local_inner_macros)]
macro_rules! __lazy_static {
    (($($vis:tt)*) static $N:ident : $T:ty = $e:expr;) => {
        #[allow(non_camel_case_types)]
        $($vis)* struct $N {}
        $($vis)* static $N: $N = $N {};
        impl core::ops::Deref for $N {
            type Target = $T;
            fn deref(&self) -> &$T {
                static mut LAZY: $crate::once::Once<$T> = $crate::once::Once::new();
                unsafe { LAZY.once(|| $e) }
            }
        }
    }
}
