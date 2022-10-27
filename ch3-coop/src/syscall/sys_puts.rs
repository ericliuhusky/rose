use crate::puts::puts;

pub fn sys_puts(buf: *const u8, len: usize) -> isize {
    let slice = unsafe {
        core::slice::from_raw_parts(buf, len)
    };
    let str = core::str::from_utf8(slice).unwrap();
    puts(str);
    len as isize
}
