use crate::mm::memory_set::KERNEL_SPACE;
use frame_allocator::{alloc, dealloc};
use page_table::{Address, Page, PA, PPN, VA};
use virtio_drivers::Hal;

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let mut ppn_base = PPN::new(0);
        for i in 0..pages {
            let frame = alloc();
            if i == 0 {
                ppn_base = PPN::new(frame);
            }
            assert_eq!(frame, ppn_base.number() + i);
        }
        ppn_base.start_addr().number()
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PA::new(pa);
        let mut ppn_base = pa.page().number();
        for _ in 0..pages {
            dealloc(ppn_base);
            ppn_base += 1;
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        KERNEL_SPACE.page_table.translate_one_addr(vaddr)
    }
}
