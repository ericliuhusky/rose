use super::*;
use core::marker::PhantomData;

/// A virtual memory address in the address space of the program.
pub type VirtAddr = usize;

/// A physical address as used for virtio.
pub type PhysAddr = usize;

/// A region of contiguous physical memory used for DMA.
#[derive(Debug)]
pub struct DMA<H: Hal> {
    paddr: usize,
    _phantom: PhantomData<H>,
}

impl<H: Hal> DMA<H> {
    pub fn new(pages: usize) -> Result<Self> {
        let paddr = H::dma_alloc(pages);
        if paddr == 0 {
            return Err(Error::DmaError);
        }
        Ok(DMA {
            paddr,
            _phantom: PhantomData::default(),
        })
    }

    pub fn vaddr(&self) -> usize {
        H::phys_to_virt(self.paddr)
    }

    /// Returns the physical page frame number.
    pub fn pfn(&self) -> u32 {
        (self.paddr >> 12) as u32
    }
}

/// The interface which a particular hardware implementation must implement.
pub trait Hal {
    /// Allocates the given number of contiguous physical pages of DMA memory for virtio use.
    fn dma_alloc(pages: usize) -> PhysAddr;
    /// Converts a physical address used for virtio to a virtual address which the program can
    /// access.
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr;
    /// Converts a virtual address which the program can access to the corresponding physical
    /// address to use for virtio.
    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr;
}
