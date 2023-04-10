use super::*;
use crate::header::VirtIOHeader;
use crate::queue::VirtQueue;
use core::hint::spin_loop;
use log::*;

/// The virtio block device is a simple virtual block device (ie. disk).
///
/// Read and write requests (and other exotic requests) are placed in the queue,
/// and serviced (probably out of order) by the device except where noted.
pub struct VirtIOBlk<'a, H: Hal> {
    header: &'static mut VirtIOHeader,
    queue: VirtQueue<'a, H>,
    capacity: usize,
}

impl<H: Hal> VirtIOBlk<'_, H> {
    /// Create a new VirtIO-Blk driver.
    pub fn new(header: &'static mut VirtIOHeader) -> Result<Self> {
        header.begin_init(|features| {
            // negotiate these flags only
            0
        });

        // read configuration space
        let config = unsafe { &mut *(header.config_space() as *mut BlkConfig) };
        info!("config: {:?}", config);
        info!(
            "found a block device of size {}KB",
            config.capacity / 2
        );

        let queue = VirtQueue::new(header, 0, 16)?;
        header.finish_init();

        Ok(VirtIOBlk {
            header,
            queue,
            capacity: config.capacity as usize,
        })
    }

    /// Acknowledge interrupt.
    pub fn ack_interrupt(&mut self) -> bool {
        self.header.ack_interrupt()
    }

    /// Read a block.
    pub fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result {
        assert_eq!(buf.len(), BLK_SIZE);
        let req = BlkReq {
            type_: ReqType::In,
            reserved: 0,
            sector: block_id as u64,
        };
        let mut resp = BlkResp::default();
        self.queue.add(&[req.as_buf()], &[buf, resp.as_buf_mut()])?;
        self.header.notify(0);
        while !self.queue.can_pop() {
            spin_loop();
        }
        self.queue.pop_used()?;
        match resp.status {
            RespStatus::Ok => Ok(()),
            _ => Err(Error::IoError),
        }
    }

    /// Read a block in a non-blocking way which means that it returns immediately.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The identifier of the block to read.
    /// * `buf` - The buffer in the memory which the block is read into.
    /// * `resp` - A mutable reference to a variable provided by the caller
    ///   which contains the status of the requests. The caller can safely
    ///   read the variable only after the request is ready.
    ///
    /// # Usage
    ///
    /// It will submit request to the virtio block device and return a token identifying
    /// the position of the first Descriptor in the chain. If there are not enough
    /// Descriptors to allocate, then it returns [Error::BufferTooSmall].
    ///
    /// After the request is ready, `resp` will be updated and the caller can get the
    /// status of the request(e.g. succeed or failed) through it. However, the caller
    /// **must not** spin on `resp` to wait for it to change. A safe way is to read it
    /// after the same token as this method returns is fetched through [VirtIOBlk::pop_used()],
    /// which means that the request has been ready.
    ///
    /// # Safety
    ///
    /// `buf` is still borrowed by the underlying virtio block device even if this
    /// method returns. Thus, it is the caller's responsibility to guarantee that
    /// `buf` is not accessed before the request is completed in order to avoid
    /// data races.
    pub unsafe fn read_block_nb(
        &mut self,
        block_id: usize,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> Result<u16> {
        assert_eq!(buf.len(), BLK_SIZE);
        let req = BlkReq {
            type_: ReqType::In,
            reserved: 0,
            sector: block_id as u64,
        };
        let token = self.queue.add(&[req.as_buf()], &[buf, resp.as_buf_mut()])?;
        self.header.notify(0);
        Ok(token)
    }

    /// Write a block.
    pub fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result {
        assert_eq!(buf.len(), BLK_SIZE);
        let req = BlkReq {
            type_: ReqType::Out,
            reserved: 0,
            sector: block_id as u64,
        };
        let mut resp = BlkResp::default();
        self.queue.add(&[req.as_buf(), buf], &[resp.as_buf_mut()])?;
        self.header.notify(0);
        while !self.queue.can_pop() {
            spin_loop();
        }
        self.queue.pop_used()?;
        match resp.status {
            RespStatus::Ok => Ok(()),
            _ => Err(Error::IoError),
        }
    }

    //// Write a block in a non-blocking way which means that it returns immediately.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The identifier of the block to write.
    /// * `buf` - The buffer in the memory containing the data to write to the block.
    /// * `resp` - A mutable reference to a variable provided by the caller
    ///   which contains the status of the requests. The caller can safely
    ///   read the variable only after the request is ready.
    ///
    /// # Usage
    ///
    /// See also [VirtIOBlk::read_block_nb()].
    ///
    /// # Safety
    ///
    /// See also [VirtIOBlk::read_block_nb()].
    pub unsafe fn write_block_nb(
        &mut self,
        block_id: usize,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> Result<u16> {
        assert_eq!(buf.len(), BLK_SIZE);
        let req = BlkReq {
            type_: ReqType::Out,
            reserved: 0,
            sector: block_id as u64,
        };
        let token = self.queue.add(&[req.as_buf(), buf], &[resp.as_buf_mut()])?;
        self.header.notify(0);
        Ok(token)
    }

    /// During an interrupt, it fetches a token of a completed request from the used
    /// ring and return it. If all completed requests have already been fetched, return
    /// Err(Error::NotReady).
    pub fn pop_used(&mut self) -> Result<u16> {
        self.queue.pop_used().map(|p| p.0)
    }

    /// Return size of its VirtQueue.
    /// It can be used to tell the caller how many channels he should monitor on.
    pub fn virt_queue_size(&self) -> u16 {
        self.queue.size()
    }
}

#[repr(C)]
#[derive(Debug)]
struct BlkConfig {
    /// Number of 512 Bytes sectors
    capacity: u64,
}

#[repr(C)]
#[derive(Debug)]
struct BlkReq {
    type_: ReqType,
    reserved: u32,
    sector: u64,
}

/// Response of a VirtIOBlk request.
#[repr(C)]
#[derive(Debug)]
pub struct BlkResp {
    status: RespStatus,
}

impl BlkResp {
    /// Return the status of a VirtIOBlk request.
    pub fn status(&self) -> RespStatus {
        self.status
    }
}

#[repr(u32)]
#[derive(Debug)]
enum ReqType {
    In = 0,
    Out = 1,
    Flush = 4,
    Discard = 11,
    WriteZeroes = 13,
}

/// Status of a VirtIOBlk request.
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RespStatus {
    /// Ok.
    Ok = 0,
    /// IoErr.
    IoErr = 1,
    /// Unsupported yet.
    Unsupported = 2,
    /// Not ready.
    _NotReady = 3,
}

impl Default for BlkResp {
    fn default() -> Self {
        BlkResp {
            status: RespStatus::_NotReady,
        }
    }
}

const BLK_SIZE: usize = 512;

unsafe impl AsBuf for BlkReq {}
unsafe impl AsBuf for BlkResp {}
