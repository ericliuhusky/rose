use crate::puts::puts;
use crate::task::TASK_MANAGER;

pub fn sys_puts(buf: *const u8, len: usize) -> isize {
    let page_table = unsafe {
        &TASK_MANAGER.current_task().page_table
    };
    let start = buf as usize;
    let end = start + len;
    let buffers = page_table.translated_byte_buffer(start..end);
    for buffer in buffers {
        let str = core::str::from_utf8(buffer).unwrap();
        puts(str);
    }
    len as isize
}
