use super::virtio_bus::VirtioHal;
use alloc::rc::Rc;
use core::cell::RefCell;
use fs::BlockDevice;
use lazy_static::*;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

lazy_static! {
    pub static ref BLOCK_DEVICE: Rc<dyn BlockDevice> = Rc::new(VirtIOBlock::new());
}

#[allow(unused)]
const VIRTIO0: usize = 0x10008000;

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
