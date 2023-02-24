use super::page_table::页表项;

pub struct 内存地址(pub usize);

impl 内存地址 {
    pub fn 页内偏移(&self) -> usize {
        self.0 & 0xfff
    }

    pub fn 页号(&self) -> usize {
        self.0 >> 12
    }

    pub fn 对齐到分页向下取整(&self) -> 内存地址 {
        内存地址(self.0 & !0xfff)
    }
    pub fn 对齐到分页向上取整(&self) -> 内存地址 {
        内存地址((self.0 + 0xfff) & !0xfff)
    }
}

#[derive(Clone)]
pub struct 内存分页 {
    pub 页号: usize,
    pub 起始地址: usize,
    pub 结尾地址: usize
}

impl 内存分页 {
    pub fn 新建(页号: usize) -> Self {
        Self { 
            页号,
            起始地址: 页号 << 12,
            结尾地址: (页号 + 1) << 12
        }
    }

    pub fn 页表项索引列表(&self) -> [usize; 3] {
        let mut vpn = self.页号;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}
