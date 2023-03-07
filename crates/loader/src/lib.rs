#![no_std]

extern crate alloc;
use alloc::vec::Vec;

pub fn read_app_num() -> usize {
    extern "C" {
        fn _app_num();
    }
    unsafe { *(_app_num as usize as *const usize) }
}

pub fn read_app_data(i: usize) -> &'static [u8] {
    extern "C" {
        fn _app_num();
    }
    let n = read_app_num();
    let _app_num_ptr = _app_num as usize as *const usize;
    unsafe {
        let start_address_ptr = _app_num_ptr.add(1);
        let start_address_list = core::slice::from_raw_parts(start_address_ptr, n + 1);
        core::slice::from_raw_parts(
            start_address_list[i] as *const u8,
            start_address_list[i + 1] - start_address_list[i],
        )
    }
}

pub fn read_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let n = read_app_num();
    extern "C" {
        fn _app_names();
    }
    let mut start_address_ptr = _app_names as usize as *const u8;
    let mut names = Vec::new();
    unsafe {
        for _ in 0..n {
            let mut end_address_ptr = start_address_ptr;
            while *end_address_ptr != b'\0' {
                end_address_ptr = end_address_ptr.add(1);
            }
            let bytes = core::slice::from_raw_parts(start_address_ptr, (end_address_ptr as usize + 1) - start_address_ptr as usize);
            let s = core::str::from_utf8(bytes).unwrap();
            names.push(s);
            start_address_ptr = end_address_ptr.add(1);
        }
    }
    (0..n)
        .find(|i|{
            names[*i] == name
        })
        .map(read_app_data)
}
