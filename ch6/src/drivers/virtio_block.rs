use alloc::sync::Arc;
use fs::BlockDevice;
use lazy_static::*;
use core::cell::RefCell;
use crate::mm::memory_set::内核地址空间;
use frame_allocator::{alloc, dealloc};
use alloc::vec::Vec;
use lazy_static::*;
use page_table::{PA, PPN, VA};
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(VirtIOBlock::new());
}

#[allow(unused)]
const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock(RefCell<VirtIOBlk<'static, VirtioHal>>);


impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .borrow_mut()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .borrow_mut()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            Self(RefCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        }
    }
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let mut ppn_base = PPN::new(0);
        for i in 0..pages {
            let frame = alloc();
            if i == 0 {
                ppn_base = PPN::new(frame);
            }
            assert_eq!(frame, ppn_base.0 + i);
        }
        ppn_base.start_addr().0
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PA::new(pa);
        let mut ppn_base = pa.page_number().0;
        for _ in 0..pages {
            dealloc(ppn_base);
            ppn_base += 1;
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        内核地址空间
            .page_table
            .translate_addr(VA::new(vaddr), VA::new(vaddr))[0]
            .0
             .0
    }
}
