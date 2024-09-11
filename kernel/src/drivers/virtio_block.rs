use alloc::rc::Rc;
use core_ext::cell::SafeCell;
use fs::BlockDevice;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

static_var! {
    BLOCK_DEVICE: &'static dyn BlockDevice = &VirtIOBlock;
}

static_var! {
    BLK: SafeCell<VirtIOBlk> = SafeCell::new(VirtIOBlk::new(
        &mut *(VIRTIO0 as *mut VirtIOHeader),
    ));
}

#[allow(unused)]
const VIRTIO0: usize = 0x10008000;

pub struct VirtIOBlock;

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        BLK.borrow_mut().read_block(block_id, buf);
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        BLK.borrow_mut().write_block(block_id, buf);
    }
}
