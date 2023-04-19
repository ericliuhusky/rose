#![no_std]
#![feature(allow_internal_unstable)]

use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            #[cfg(feature = "user")]
            sys_call::putchar(c as usize);
            #[cfg(not(feature = "user"))]
            sbi_call::putchar(c as usize);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::_print(format_args_nl!($($arg)*));
    }};
}
