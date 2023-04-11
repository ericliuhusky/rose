use crate::mm::memory_set::KERNEL_SPACE;
use frame_allocator::{alloc, dealloc, alloc_more};
use page_table::{Address, Page, PA, PPN, VA};
use virtio_drivers::Hal;

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let ppns = alloc_more(pages);
        let ppn_base = PPN::new(*ppns.last().unwrap());
        ppn_base.start_addr().number()
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        KERNEL_SPACE.page_table.translate_one_addr(vaddr)
    }
}