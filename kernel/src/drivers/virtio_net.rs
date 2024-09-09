use super::virtio_bus::VirtioHal;
use alloc::rc::Rc;
use core_ext::cell::SafeCell;
use virtio_drivers::{VirtIOHeader, VirtIONet};

const VIRTIO8: usize = 0x10007000;

static_var! {
    NET_DEVICE: Rc<dyn NetDevice> = Rc::new(VirtIONetWrapper::new());
}

pub trait NetDevice {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
}

pub struct VirtIONetWrapper(SafeCell<VirtIONet<VirtioHal>>);

impl NetDevice for VirtIONetWrapper {
    fn transmit(&self, data: &[u8]) {
        self.0.borrow_mut().send(data);
    }

    fn receive(&self, data: &mut [u8]) -> usize {
        self.0.borrow_mut().recv(data)
    }
}

impl VirtIONetWrapper {
    pub fn new() -> Self {
        unsafe {
            let virtio = VirtIONet::<VirtioHal>::new(&mut *(VIRTIO8 as *mut VirtIOHeader));
            VirtIONetWrapper(SafeCell::new(virtio))
        }
    }
}
