use alloc::vec::Vec;

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

pub fn 通过名称读取应用数据(应用名称: &str) -> Option<&'static [u8]> {
    let 应用数目 = 读取应用数目();
    extern "C" {
        fn _app_names();
    }
    let mut 应用名称起始地址指针 = _app_names as usize as *const u8;
    let mut 应用名称列表 = Vec::new();
    unsafe {
        for _ in 0..应用数目 {
            let mut 应用名称结尾地址指针 = 应用名称起始地址指针;
            while *应用名称结尾地址指针 != b'\0' {
                应用名称结尾地址指针 = 应用名称结尾地址指针.add(1);
            }
            let 字节数组 = core::slice::from_raw_parts(应用名称起始地址指针, (应用名称结尾地址指针 as usize + 1) - 应用名称起始地址指针 as usize);
            let 字节串 = core::str::from_utf8(字节数组).unwrap();
            应用名称列表.push(字节串);
            应用名称起始地址指针 = 应用名称结尾地址指针.add(1);
        }
    }
    (0..应用数目)
        .find(|应用索引| {
            应用名称列表[*应用索引] == 应用名称
        })
        .map(读取应用数据)
}
