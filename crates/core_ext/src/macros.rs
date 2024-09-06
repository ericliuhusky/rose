#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::io::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::io::_print(format_args!($($arg)*));
        $crate::println!()
    }};
}

/// # 超好用的`static`变量计算属性
/// ## `A::set()`设置值，直接使用`A`访问值
/// - 访问`static`变量时不需要`unsafe`
/// - 允许初始化有非`const`函数，在访问时进行且仅进行一次初始化（懒加载）
/// - 用`SafeCell`包裹提供内部可变性，访问`static mut`也不需要`unsafe`
/// - 实现`Deref`以支持`.`访问
#[macro_export]
macro_rules! static_var {
    ($N:ident : $T:ty = $e:expr;) => {
        #[allow(non_camel_case_types)]
        pub struct $N {
            data: $crate::cell::SafeCell<core::mem::MaybeUninit<$T>>,
            once_flag: $crate::cell::SafeCell<bool>,
        }
        pub static $N: $N = $N {
            data: $crate::cell::SafeCell::new(core::mem::MaybeUninit::uninit()),
            once_flag: $crate::cell::SafeCell::new(true),
        };
        impl $N {
            fn lazy_init() {
                unsafe {
                    if (*$N.once_flag.get()) {
                        (*$N.once_flag.get()) = false;
                        (*$N.data.get()).write($e);
                    }
                }
            }

            fn get() -> &'static $T {
                Self::lazy_init();
                unsafe { &*(*$N.data.get()).as_ptr() }
            }

            pub fn set(val: $T) {
                unsafe {
                    (*$N.once_flag.get()) = false;
                    (*$N.data.get()).write(val);
                }
            }
        }
        impl core::ops::Deref for $N {
            type Target = $T;
            fn deref(&self) -> &Self::Target {
                Self::get()
            }
        }
    };
}
