use core::mem::{size_of, MaybeUninit};

use super::*;
use bitflags::*;
use core::hint::spin_loop;
use log::*;

/// The virtio network device is a virtual ethernet card.
///
/// It has enhanced rapidly and demonstrates clearly how support for new
/// features are added to an existing device.
/// Empty buffers are placed in one virtqueue for receiving packets, and
/// outgoing packets are enqueued into another for transmission in that order.
/// A third command queue is used to control advanced filtering features.
pub struct VirtIONet<H: Hal> {
    header: &'static mut VirtIOHeader,
    mac: EthernetAddress,
    recv_queue: VirtQueue<H>,
    send_queue: VirtQueue<H>,
}

impl<H: Hal> VirtIONet<H> {
    /// Create a new VirtIO-Net driver.
    pub fn new(header: &'static mut VirtIOHeader) -> Result<Self> {
        header.init();
        // read configuration space
        let config = unsafe { &mut *(header.config_space() as *mut Config) };
        let mac = config.mac;
        debug!("Got MAC={:?}, status={:?}", mac, config.status);

        let recv_queue = VirtQueue::new(header, QUEUE_RECEIVE, 16)?;
        let send_queue = VirtQueue::new(header, QUEUE_TRANSMIT, 16)?;

        Ok(VirtIONet {
            header,
            mac,
            recv_queue,
            send_queue,
        })
    }

    /// Acknowledge interrupt.
    pub fn ack_interrupt(&mut self) -> bool {
        self.header.ack_interrupt()
    }

    /// Get MAC address.
    pub fn mac(&self) -> EthernetAddress {
        self.mac
    }

    /// Whether can send packet.
    pub fn can_send(&self) -> bool {
        self.send_queue.available_desc() >= 2
    }

    /// Whether can receive packet.
    pub fn can_recv(&self) -> bool {
        self.recv_queue.can_pop()
    }

    /// Receive a packet.
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut header = MaybeUninit::<Header>::uninit();
        let header_buf = unsafe { (*header.as_mut_ptr()).as_buf_mut() };
        self.recv_queue.add(&[], &[header_buf, buf])?;
        self.header.notify(QUEUE_RECEIVE as u32);
        while !self.recv_queue.can_pop() {
            spin_loop();
        }

        let (_, len) = self.recv_queue.pop_used()?;
        // let header = unsafe { header.assume_init() };
        Ok(len as usize - size_of::<Header>())
    }

    /// Send a packet.
    pub fn send(&mut self, buf: &[u8]) -> Result {
        let header = unsafe { MaybeUninit::<Header>::zeroed().assume_init() };
        self.send_queue.add(&[header.as_buf(), buf], &[])?;
        self.header.notify(QUEUE_TRANSMIT as u32);
        while !self.send_queue.can_pop() {
            spin_loop();
        }
        self.send_queue.pop_used()?;
        Ok(())
    }
}

bitflags! {
    struct Status: u16 {
        const LINK_UP = 1;
        const ANNOUNCE = 2;
    }
}

bitflags! {
    struct InterruptStatus : u32 {
        const USED_RING_UPDATE = 1 << 0;
        const CONFIGURATION_CHANGE = 1 << 1;
    }
}

#[repr(C)]
#[derive(Debug)]
struct Config {
    mac: EthernetAddress,
    status: Status,
}

type EthernetAddress = [u8; 6];

// virtio 5.1.6 Device Operation
#[repr(C)]
#[derive(Debug)]
struct Header {
    flags: Flags,
    gso_type: u8,
    hdr_len: u16, // cannot rely on this
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16,
    // payload starts from here
}

unsafe impl AsBuf for Header {}

bitflags! {
    struct Flags: u8 {
        const NEEDS_CSUM = 1;
        const DATA_VALID = 2;
        const RSC_INFO   = 4;
    }
}

const QUEUE_RECEIVE: usize = 0;
const QUEUE_TRANSMIT: usize = 1;
