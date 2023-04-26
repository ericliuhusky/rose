pub fn check_sum(ptr: *const u8, mut len: usize, mut sum: u32) -> u16 {
    let mut ptr = ptr as *const u16;

    while len > 1 {
        sum += unsafe { *ptr } as u32;
        unsafe {
            ptr = ptr.offset(1);
        }
        len -= 2;
    }

    if len == 1 {
        sum += unsafe { *(ptr as *const u8) } as u32;
    }

    fn fold(mut sum: u32) -> u16 {
        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }
        !sum as u16
    }
    fold(sum)
}
