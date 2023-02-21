use super::PageTableEntry;


#[derive(Copy, Clone)]
pub struct 物理页(pub usize);

#[derive(Copy, Clone)]
pub struct 虚拟页(pub usize);



pub fn 将地址转为页号并向下取整(v: usize) -> usize {
    v >> 12
}
pub fn 将地址转为页号并向上取整(v: usize) -> usize {
    (v + (1 << 12) - 1) >> 12
}
pub fn 页内偏移(v: usize) -> usize {
    v & 0xfff
}

impl 虚拟页 {
    pub fn 地址所在的虚拟页(虚拟地址: usize) -> Self {
        虚拟页(将地址转为页号并向下取整(虚拟地址))
    }

    pub fn 页表项索引列表(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

    pub fn 起始地址(&self) -> usize {
        self.0 << 12
    }

    pub fn 下一页(&self) -> Self {
        虚拟页(self.0 + 1)
    }
}

impl 物理页 {
    fn 起始地址(&self) -> usize {
        self.0 << 12
    }

    pub fn 读取页表项列表(&self) -> &'static mut [PageTableEntry] {
        let pa = self.起始地址();
        unsafe { core::slice::from_raw_parts_mut(pa as *mut PageTableEntry, 512) }
    }
    pub fn 读取字节列表(&self) -> &'static mut [u8] {
        let pa = self.起始地址();
        unsafe { core::slice::from_raw_parts_mut(pa as *mut u8, 4096) }
    }
    pub fn 以某种类型来读取<T>(&self) -> &'static mut T {
        let pa = self.起始地址();
        unsafe { (pa as *mut T).as_mut().unwrap() }
    }
}
