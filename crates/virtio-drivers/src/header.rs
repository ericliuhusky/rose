use crate::PAGE_SIZE;
use bitflags::*;
use volatile::{ReadOnly, Volatile, WriteOnly};

const MAGIC_VALUE: u32 = 0x7472_6976;

/// MMIO Device Legacy Register Interface.
///
/// Ref: 4.2.4 Legacy interface
#[repr(C)]
pub struct VirtIOHeader {
    _magic: u32,
    _version: u32,
    _device_id: u32,
    _vendor_id: u32,
    device_features: ReadOnly<u32>,

    /// Device (host) features word selection
    device_features_sel: WriteOnly<u32>,

    /// Reserved
    __r1: [u32; 2],

    /// Flags representing device features understood and activated by the driver
    driver_features: WriteOnly<u32>,

    /// Activated (guest) features word selection
    driver_features_sel: WriteOnly<u32>,

    /// Guest page size
    ///
    /// The driver writes the guest page size in bytes to the register during
    /// initialization, before any queues are used. This value should be a
    /// power of 2 and is used by the device to calculate the Guest address
    /// of the first queue page (see QueuePFN).
    guest_page_size: WriteOnly<u32>,

    /// Reserved
    __r2: u32,

    /// Virtual queue index
    ///
    /// Writing to this register selects the virtual queue that the following
    /// operations on the QueueNumMax, QueueNum, QueueAlign and QueuePFN
    /// registers apply to. The index number of the first queue is zero (0x0).
    queue_sel: WriteOnly<u32>,

    /// Maximum virtual queue size
    ///
    /// Reading from the register returns the maximum size of the queue the
    /// device is ready to process or zero (0x0) if the queue is not available.
    /// This applies to the queue selected by writing to QueueSel and is
    /// allowed only when QueuePFN is set to zero (0x0), so when the queue is
    /// not actively used.
    queue_num_max: ReadOnly<u32>,

    /// Virtual queue size
    ///
    /// Queue size is the number of elements in the queue. Writing to this
    /// register notifies the device what size of the queue the driver will use.
    /// This applies to the queue selected by writing to QueueSel.
    queue_num: WriteOnly<u32>,

    /// Used Ring alignment in the virtual queue
    ///
    /// Writing to this register notifies the device about alignment boundary
    /// of the Used Ring in bytes. This value should be a power of 2 and
    /// applies to the queue selected by writing to QueueSel.
    queue_align: WriteOnly<u32>,

    /// Guest physical page number of the virtual queue
    ///
    /// Writing to this register notifies the device about location of the
    /// virtual queue in the Guest’s physical address space. This value is
    /// the index number of a page starting with the queue Descriptor Table.
    /// Value zero (0x0) means physical address zero (0x00000000) and is illegal.
    /// When the driver stops using the queue it writes zero (0x0) to this
    /// register. Reading from this register returns the currently used page
    /// number of the queue, therefore a value other than zero (0x0) means that
    /// the queue is in use. Both read and write accesses apply to the queue
    /// selected by writing to QueueSel.
    queue_pfn: Volatile<u32>,

    _queue_ready: u32,

    /// Reserved
    __r3: [u32; 2],

    /// Queue notifier
    queue_notify: WriteOnly<u32>,

    /// Reserved
    __r4: [u32; 3],

    /// Interrupt status
    interrupt_status: ReadOnly<u32>,

    /// Interrupt acknowledge
    interrupt_ack: WriteOnly<u32>,

    /// Reserved
    __r5: [u32; 2],

    /// Device status
    ///
    /// Reading from this register returns the current device status flags.
    /// Writing non-zero values to this register sets the status flags,
    /// indicating the OS/driver progress. Writing zero (0x0) to this register
    /// triggers a device reset. The device sets QueuePFN to zero (0x0) for
    /// all queues in the device. Also see 3.1 Device Initialization.
    status: Volatile<u32>,

    /// Reserved
    __r6: [u32; 3],

    _queue_desc_low: WriteOnly<u32>,
    _queue_desc_high: WriteOnly<u32>,

    /// Reserved
    __r7: [u32; 2],

    _queue_avail_low: u32,
    _queue_avail_high: u32,

    /// Reserved
    __r8: [u32; 2],

    _queue_used_low: u32,
    _queue_used_high: u32,

    /// Reserved
    __r9: [u32; 21],

    _config_generation: u32,
}

impl VirtIOHeader {
    /// Begin initializing the device.
    ///
    /// Ref: virtio 3.1.1 Device Initialization
    pub fn begin_init(&mut self, negotiate_features: impl FnOnce(u64) -> u64) {
        let features = self.read_device_features();
        self.write_driver_features(negotiate_features(features));

        self.guest_page_size.write(PAGE_SIZE as u32);
    }

    /// Finish initializing the device.
    pub fn finish_init(&mut self) {
        self.status.write(DRIVER_OK);
    }

    /// Read device features.
    fn read_device_features(&mut self) -> u64 {
        self.device_features_sel.write(0); // device features [0, 32)
        let mut device_features_bits = self.device_features.read().into();
        self.device_features_sel.write(1); // device features [32, 64)
        device_features_bits += (self.device_features.read() as u64) << 32;
        device_features_bits
    }

    /// Write device features.
    fn write_driver_features(&mut self, driver_features: u64) {
        self.driver_features_sel.write(0); // driver features [0, 32)
        self.driver_features.write(driver_features as u32);
        self.driver_features_sel.write(1); // driver features [32, 64)
        self.driver_features.write((driver_features >> 32) as u32);
    }

    /// Set queue.
    pub fn queue_set(&mut self, queue: u32, size: u32, align: u32, pfn: u32) {
        self.queue_sel.write(queue);
        self.queue_num.write(size);
        self.queue_align.write(align);
        self.queue_pfn.write(pfn);
    }

    /// Get guest physical page number of the virtual queue.
    pub fn queue_physical_page_number(&mut self, queue: u32) -> u32 {
        self.queue_sel.write(queue);
        self.queue_pfn.read()
    }

    /// Whether the queue is in used.
    pub fn queue_used(&mut self, queue: u32) -> bool {
        self.queue_physical_page_number(queue) != 0
    }

    /// Get the max size of queue.
    pub fn max_queue_size(&self) -> u32 {
        self.queue_num_max.read()
    }

    /// Notify device.
    pub fn notify(&mut self, queue: u32) {
        self.queue_notify.write(queue);
    }

    /// Acknowledge interrupt and return true if success.
    pub fn ack_interrupt(&mut self) -> bool {
        let interrupt = self.interrupt_status.read();
        if interrupt != 0 {
            self.interrupt_ack.write(interrupt);
            true
        } else {
            false
        }
    }

    /// Get the pointer to config space (at offset 0x100)
    pub fn config_space(&self) -> *mut u64 {
        (self as *const _ as usize + CONFIG_SPACE_OFFSET) as _
    }
}

const DRIVER_OK: u32 = 4;

const CONFIG_SPACE_OFFSET: usize = 0x100;
