use core::marker::PhantomData;

/// A virtual memory address in the address space of the program.
pub type VirtAddr = usize;

/// A physical address as used for virtio.
pub type PhysAddr = usize;

/// A region of contiguous physical memory used for DMA.
#[derive(Debug)]
pub struct DMA<H: Hal> {
    pub paddr: usize,
    _phantom: PhantomData<H>,
}

impl<H: Hal> DMA<H> {
    pub fn new(pages: usize) -> Self {
        let paddr = H::dma_alloc(pages);
        Self {
            paddr,
            _phantom: PhantomData::default(),
        }
    }

    /// Returns the physical page frame number.
    pub fn ppn(&self) -> u32 {
        (self.paddr >> 12) as u32
    }
}

/// The interface which a particular hardware implementation must implement.
pub trait Hal {
    /// Allocates the given number of contiguous physical pages of DMA memory for virtio use.
    fn dma_alloc(pages: usize) -> PhysAddr;
}
