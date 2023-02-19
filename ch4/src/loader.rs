pub fn 读取应用数目() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn 读取应用数据(应用索引: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let 应用数目 = 读取应用数目();
    let 应用数目指针 = _num_app as usize as *const usize;
    unsafe {
        let 应用数据起始地址指针 = 应用数目指针.add(1);
        let 应用数据起始地址列表 = core::slice::from_raw_parts(应用数据起始地址指针, 应用数目 + 1);
        core::slice::from_raw_parts(
            应用数据起始地址列表[应用索引] as *const u8,
            应用数据起始地址列表[应用索引 + 1] - 应用数据起始地址列表[应用索引],
        )
    }
}
