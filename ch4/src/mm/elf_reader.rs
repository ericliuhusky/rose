use core::ops::Range;
use alloc::vec::Vec;

pub struct Elf文件<'a> { 
    头: &'a 头,
    程序段列表: Vec<程序段<'a>>
}

impl<'a> Elf文件<'a> {
    pub fn 解析(数据: &'a [u8]) -> Self {
        let 头: &头 = 以某种类型来读取(数据);
        assert_eq!(头.魔数, [0x7f, b'E', b'L', b'F']);
        assert_eq!(头.类型32或64位, 0x2);

        let mut 程序段列表 = Vec::new();
        for i in 0..头.程序段头数目 {
            let 头 = Self::程序段头(数据, 头, i);
            let 数据 = Self::程序段数据(数据, 头);
            程序段列表.push(程序段 { 头, 数据 });
        }
        Self { 
            头,
            程序段列表
        }
    }

    fn 程序段头(数据: &'a [u8], 头: &头, 程序段头索引: u16) -> &'a 程序段头 {
        let 起始 = 头.程序段头偏移 + 程序段头索引 as usize * 头.程序段头大小 as usize;
        let 结尾 = 起始 + 头.程序段头大小 as usize;
        let 数据 = &数据[起始..结尾];
        以某种类型来读取(数据)
    }

    fn 程序段数据(数据: &'a [u8], 头: &程序段头) -> &'a [u8] {
        let 起始 = 头.程序段数据偏移;
        let 结尾 = 起始 + 头.文件大小;
        &数据[起始..结尾]
    }

    pub fn 入口地址(&self) -> usize {
        self.头.入口地址
    }

    pub fn 程序段列表(&self) -> Vec<&程序段> {
        let mut ps = Vec::new();
        for p in &self.程序段列表 {
            if p.需要被加载() {
                ps.push(p);
            }
        }
        ps
    }

    pub fn 最后一个程序段的结尾虚拟地址(&self) -> usize {
        let 最后一个程序段 = self.程序段列表.last().unwrap();
        最后一个程序段.虚拟地址范围().end
    }
}

#[repr(C)]
struct 头 {
    魔数: [u8; 4],
    类型32或64位: u8,
    _未使用的占位字节列表1: [u8; 19],
    入口地址: usize,
    程序段头偏移: usize,
    _未使用的占位字节列表2: [u8; 14],
    程序段头大小: u16,
    程序段头数目: u16
}

#[repr(C)]
struct 程序段头 {
    类型: u32,
    _未使用的占位字节列表1: [u8; 4],
    程序段数据偏移: usize,
    起始虚拟地址: usize,
    _未使用的占位字节列表2: [u8; 8],
    文件大小: usize,
    内存大小: usize
}

pub struct 程序段<'a> {
    头: &'a 程序段头,
    pub 数据: &'a [u8]
}

impl<'a> 程序段<'_> {
    pub fn 虚拟地址范围(&self) -> Range<usize> {
        let 起始虚拟地址 = self.头.起始虚拟地址;
        let 结尾虚拟地址 = 起始虚拟地址 + self.头.内存大小;
        起始虚拟地址..结尾虚拟地址
    }

    fn 需要被加载(&self) -> bool {
        self.头.类型 == 0x1
    }
}

fn 以某种类型来读取<T>(字节串: &[u8]) -> &T {
    unsafe {
        &*(字节串.as_ptr() as *const T)
    }
}
