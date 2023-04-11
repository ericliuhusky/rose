use crate::mm::memory_set::{DMA_END_ADDR, DMA_START_ADDR, KERNEL_SPACE};
use core::cell::RefCell;
use page_table::{Address, Page, PA, PPN};
use virtio_drivers::Hal;

struct DMAAllocator {
    current: usize,
    end: usize,
}

impl DMAAllocator {
    fn new(start_pa: usize, end_pa: usize) -> Self {
        Self {
            current: PA::new(start_pa).align_to_upper().page().number(),
            end: PA::new(end_pa).align_to_lower().page().number(),
        }
    }

    fn alloc(&mut self, page_num: usize) -> usize {
        if self.current == self.end {
            panic!()
        }
        let ppn = self.current;
        self.current += page_num;
        ppn
    }
}

lazy_static::lazy_static! {
    static ref DMA_ALLOCATOR: RefCell<DMAAllocator> = RefCell::new(DMAAllocator::new(DMA_START_ADDR, DMA_END_ADDR));
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let ppn = DMA_ALLOCATOR.borrow_mut().alloc(pages);
        PPN::new(ppn).start_addr().number()
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        KERNEL_SPACE.page_table.translate_one_addr(vaddr)
    }
}
