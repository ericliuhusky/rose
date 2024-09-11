pub mod virtio_block;
pub mod virtio_net;

use virtio_block::VirtIOBlock;

pub fn init() {
    fs::init(&VirtIOBlock);
}
