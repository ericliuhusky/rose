// check sum function
pub fn check_sum(addr:*mut u8, len:u32, sum: u32) -> u16 {
    let mut sum:u32 = sum;
    let mut nleft = len;
    let mut w = addr as *const u16;
    
     while nleft > 1 {
        sum += unsafe{ *w as u32 };
        w = (w as usize + 2) as *mut u16;
        nleft -= 2;

        if sum > 0xffff {
            sum = (sum & 0xFFFF) + (sum >> 16);
            sum = sum + (sum >> 16);
        }
     }

     if nleft == 1 {
        sum += unsafe { *(w as *const u8) as u32};
     }


     sum = (sum & 0xFFFF) + (sum >> 16);
     sum = sum + (sum >> 16);

     let answer:u16 = !sum as u16;

     answer
}