pub struct 内存地址(pub usize);

impl 内存地址 {
    pub fn 页内偏移(&self) -> usize {
        self.0 & 0xfff
    }

    pub fn 页号(&self) -> usize {
        self.0 >> 12
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
}
