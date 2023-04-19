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
