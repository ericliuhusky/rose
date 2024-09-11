use fs::BlockDevice;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

static_var! {
    BLOCK_DEVICE: &'static dyn BlockDevice = &VirtIOBlock;
}

static_var! {
    BLK: VirtIOBlk = VirtIOBlk::new(
        &mut *(VIRTIO0 as *mut VirtIOHeader),
    );
}

#[allow(unused)]
const VIRTIO0: usize = 0x10008000;

pub struct VirtIOBlock;

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        BLK::get().read_block(block_id, buf);
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        BLK::get().write_block(block_id, buf);
    }
}
