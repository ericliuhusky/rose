use core::ops::Range;
use alloc::vec::Vec;

pub struct Elf文件<'a> { 
    头: &'a 头,
    程序段列表: Vec<程序段<'a>>
}

impl<'a> Elf文件<'a> {
    pub fn 解析(数据: &'a [u8]) -> Self {
        let 头 = unsafe {
            &*(数据 as *const [u8] as *const 头)
        };
        // 确保是elf格式的可执行文件
        assert_eq!(头.魔数, [0x7f, b'E', b'L', b'F']);
        // 确保是64位
        assert_eq!(头.类型32或64位, 0x2);

        let 程序段列表 = (0..头.程序段头数目)
            .map(|程序段头索引| {
                let 程序段头起始 = 头.程序段头偏移 + 程序段头索引 as usize * 头.程序段头大小 as usize;
                let 程序段头结尾 = 程序段头起始 + 头.程序段头大小 as usize;
                let 程序段头 = unsafe {
                    &*(&数据[程序段头起始..程序段头结尾] as *const [u8] as *const 程序段头)
                };
                let 程序段数据起始 = 程序段头.程序段数据偏移;
                let 程序段数据结尾 = 程序段数据起始 + 程序段头.文件大小;
                程序段 { 
                    头: 程序段头, 
                    数据: &数据[程序段数据起始..程序段数据结尾] 
                }
            })
            .collect();
        Self { 
            头,
            程序段列表
        }
    }

    pub fn 入口地址(&self) -> usize {
        self.头.入口地址
    }

    pub fn 程序段列表(&self) -> Vec<&程序段> {
        self.程序段列表
            .iter()
            .filter(|程序段| {
                程序段.需要被加载()
            })
            .collect()
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
        self.头.起始虚拟地址..self.头.起始虚拟地址 + self.头.内存大小
    }

    fn 需要被加载(&self) -> bool {
        self.头.类型 == 0x1
    }
}
