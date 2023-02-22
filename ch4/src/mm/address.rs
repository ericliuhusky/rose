use crate::mm::page_table::PageTableEntry;


#[derive(Copy, Clone)]
pub struct 物理页(pub usize);

#[derive(Copy, Clone)]
pub struct 虚拟页(pub usize);



pub fn 将地址转为页号并向下取整(v: usize) -> usize {
    v >> 12
}
pub fn 将地址转为页号并向上取整(v: usize) -> usize {
    (v + 0xfff) >> 12
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
}

impl 物理页 {
    fn 起始地址(&self) -> usize {
        self.0 << 12
    }

    pub fn 读取页表项列表(&self) -> &'static mut [PageTableEntry] {
        let pa = self.起始地址();
        unsafe {
            &mut *(pa as *mut [PageTableEntry; 512])
        }
    }
}
