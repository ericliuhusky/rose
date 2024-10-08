use core::mem::size_of;
use core::slice;
use core::sync::atomic::{fence, Ordering};

use super::*;
use crate::header::VirtIOHeader;
use bitflags::*;

use crate::volatile::Volatile;

/// The mechanism for bulk data transport on virtio devices.
///
/// Each device can have zero or more virtqueues.
pub struct VirtQueue{
    /// Descriptor table
    desc: &'static mut [Descriptor],
    /// Available ring
    avail: &'static mut AvailRing,
    /// Used ring
    used: &'static mut UsedRing,

    /// The size of the queue.
    ///
    /// This is both the number of descriptors, and the number of slots in the available and used
    /// rings.
    queue_size: u16,
    /// The number of used queues.
    num_used: u16,
    /// The head desc index of the free list.
    free_head: u16,
    avail_idx: u16,
    last_used_idx: u16,
}

impl VirtQueue {
    /// Create a new VirtQueue.
    pub fn new(header: &mut VirtIOHeader, idx: usize, size: u16, _type: &str) -> Self {
        let layout = VirtQueueLayout::new(size);
        // Allocate contiguous pages.

        let dma_start_pa: usize;
        match _type {
            "blk" => {
                dma_start_pa = 0x877f8000;
            }
            "net1" => {
                dma_start_pa = 0x877fa000;
            }
            "net2" => {
                dma_start_pa = 0x877fc000;
            }
            _ => panic!()
        }
        let dma_ppn = dma_start_pa >> 12;

        header.queue_set(idx as u32, size as u32, PAGE_SIZE as u32, dma_ppn as u32);

        let desc =
            unsafe { slice::from_raw_parts_mut(dma_start_pa as *mut Descriptor, size as usize) };
        let avail = unsafe { &mut *((dma_start_pa + layout.avail_offset) as *mut AvailRing) };
        let used = unsafe { &mut *((dma_start_pa + layout.used_offset) as *mut UsedRing) };

        // Link descriptors together.
        for i in 0..(size - 1) {
            desc[i as usize].next = i + 1;
        }

        Self {
            desc,
            avail,
            used,
            queue_size: size,
            num_used: 0,
            free_head: 0,
            avail_idx: 0,
            last_used_idx: 0,
        }
    }

    /// Add buffers to the virtqueue, return a token.
    ///
    /// Ref: linux virtio_ring.c virtqueue_add
    pub fn add(&mut self, inputs: &[&[u8]], outputs: &[&mut [u8]]) -> u16 {
        // allocate descriptors from free list
        let head = self.free_head;
        let mut last = self.free_head;
        for input in inputs.iter() {
            let desc = &mut self.desc[self.free_head as usize];
            desc.set_buf(input);
            desc.flags = DescFlags::NEXT;
            last = self.free_head;
            self.free_head = desc.next;
        }
        for output in outputs.iter() {
            let desc = &mut self.desc[self.free_head as usize];
            desc.set_buf(output);
            desc.flags = DescFlags::NEXT | DescFlags::WRITE;
            last = self.free_head;
            self.free_head = desc.next;
        }
        // set last_elem.next = NULL
        {
            let desc = &mut self.desc[last as usize];
            let mut flags = desc.flags;
            flags.remove(DescFlags::NEXT);
            desc.flags = flags;
        }
        self.num_used += (inputs.len() + outputs.len()) as u16;

        let avail_slot = self.avail_idx & (self.queue_size - 1);
        self.avail.ring[avail_slot as usize] = head;

        // write barrier
        fence(Ordering::SeqCst);

        // increase head of avail ring
        self.avail_idx = self.avail_idx.wrapping_add(1);
        self.avail.idx = self.avail_idx;
        head
    }

    /// Whether there is a used element that can pop.
    pub fn can_pop(&self) -> bool {
        self.last_used_idx != self.used.idx.read()
    }

    /// The number of free descriptors.
    pub fn available_desc(&self) -> usize {
        (self.queue_size - self.num_used) as usize
    }

    /// Recycle descriptors in the list specified by head.
    ///
    /// This will push all linked descriptors at the front of the free list.
    fn recycle_descriptors(&mut self, mut head: u16) {
        let origin_free_head = self.free_head;
        self.free_head = head;
        loop {
            let desc = &mut self.desc[head as usize];
            let flags = desc.flags;
            self.num_used -= 1;
            if flags.contains(DescFlags::NEXT) {
                head = desc.next;
            } else {
                desc.next = origin_free_head;
                return;
            }
        }
    }

    /// Get a token from device used buffers, return (token, len).
    ///
    /// Ref: linux virtio_ring.c virtqueue_get_buf_ctx
    pub fn pop_used(&mut self) -> (u16, u32) {
        // read barrier
        fence(Ordering::SeqCst);

        let last_used_slot = self.last_used_idx & (self.queue_size - 1);
        let index = self.used.ring[last_used_slot as usize].id as u16;
        let len = self.used.ring[last_used_slot as usize].len;

        self.recycle_descriptors(index);
        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        (index, len)
    }
}

/// The inner layout of a VirtQueue.
///
/// Ref: 2.6.2 Legacy Interfaces: A Note on Virtqueue Layout
struct VirtQueueLayout {
    avail_offset: usize,
    used_offset: usize,
    size: usize,
}

impl VirtQueueLayout {
    fn new(queue_size: u16) -> Self {
        let queue_size = queue_size as usize;
        let desc = size_of::<Descriptor>() * queue_size;
        let avail = size_of::<AvailRing>();
        let used = size_of::<UsedRing>();
        VirtQueueLayout {
            avail_offset: desc,
            used_offset: align_up(desc + avail),
            size: align_up(desc + avail) + align_up(used),
        }
    }
}

#[repr(C, align(16))]
struct Descriptor {
    addr: u64,
    len: u32,
    flags: DescFlags,
    next: u16,
}

impl Descriptor {
    fn set_buf(&mut self, buf: &[u8]) {
        self.addr = buf.as_ptr() as usize as u64;
        self.len = buf.len() as u32;
    }
}

bitflags! {
    /// Descriptor flags
    struct DescFlags: u16 {
        const NEXT = 1;
        const WRITE = 2;
    }
}

/// The driver uses the available ring to offer buffers to the device:
/// each ring entry refers to the head of a descriptor chain.
/// It is only written by the driver and read by the device.
#[repr(C)]
struct AvailRing {
    _flags: u16,
    /// A driver MUST NOT decrement the idx.
    idx: u16,
    ring: [u16; 16],
}

/// The used ring is where the device returns buffers once it is done with them:
/// it is only written to by the device, and read by the driver.
#[repr(C)]
struct UsedRing {
    _flags: u16,
    idx: Volatile<u16>,
    ring: [UsedElem; 16],
}

#[repr(C)]
struct UsedElem {
    id: u32,
    len: u32,
}
