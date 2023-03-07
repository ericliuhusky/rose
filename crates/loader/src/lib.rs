#![no_std]

pub fn read_app_num() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { *(_num_app as usize as *const usize) }
}

pub fn read_app_data(i: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let n = read_app_num();
    let _num_app_ptr = _num_app as usize as *const usize;
    unsafe {
        let start_address_ptr = _num_app_ptr.add(1);
        let start_address_list = core::slice::from_raw_parts(start_address_ptr, n + 1);
        core::slice::from_raw_parts(
            start_address_list[i] as *const u8,
            start_address_list[i + 1] - start_address_list[i],
        )
    }
}
