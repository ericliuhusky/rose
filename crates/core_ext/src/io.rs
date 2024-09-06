use crate::self_impl;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self_impl().putchar(c);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}
