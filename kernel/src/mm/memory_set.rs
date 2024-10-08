use alloc::vec::Vec;
use core::ops::Range;
use elf_reader::ElfFile;
use frame_allocator::FrameAllocator;

pub const AVAILABLE_MEMORY_END: usize = DMA_START_ADDR;
pub const DMA_SIZE: usize = 6 * 0x1000;
pub const DMA_START_ADDR: usize = DMA_END_ADDR - DMA_SIZE;
pub const DMA_END_ADDR: usize = KERNEL_STACK_START_ADDR;
pub const KERNEL_STACK_SIZE: usize = 0x2000;
pub const KERNEL_STACK_START_ADDR: usize = KERNEL_HEAP_START_ADDR - KERNEL_STACK_SIZE;
pub const KERNEL_STACK_END_ADDR: usize = KERNEL_HEAP_START_ADDR;
pub const KERNEL_STACK_TOP: usize = KERNEL_STACK_END_ADDR;
pub const KERNEL_HEAP_SIZE: usize = 0x800000;
pub const KERNEL_HEAP_START_ADDR: usize = KERNEL_HEAP_END_ADDR - KERNEL_HEAP_SIZE;
pub const KERNEL_HEAP_END_ADDR: usize = MEMORY_END;
pub const MEMORY_END: usize = 0x88000000;
pub const USER_STACK_SIZE: usize = 0x2000;
pub const USER_STACK_START_ADDR: usize = HIGH_START_ADDR;
pub const USER_STACK_END_ADDR: usize = USER_STACK_START_ADDR + 10 * USER_STACK_SIZE;
pub const USER_HEAP_SIZE: usize = 0x80000;
pub const USER_HEAP_START_ADDR: usize = USER_HEAP_END_ADDR - USER_HEAP_SIZE;
pub const USER_HEAP_END_ADDR: usize = HIGH_END_ADDR;

extern "C" {
    fn skernel();
    fn ekernel();
    fn strampoline();
    fn etrampoline();
}

use page_table::VA;
use page_table::{Address, Page, SV39PageTable, HIGH_END_ADDR, HIGH_START_ADDR};

trait Space {
    fn new_bare() -> Self;

    fn page_table(&mut self) -> &mut SV39PageTable<FrameAllocator>;

    fn map(&mut self, segment: Segment) {
        for vpn in segment.vpn_range() {
            self.page_table()
                .map(vpn, segment.identical, segment.user_accessible);
        }
    }
}

pub struct UserSpace {
    pub page_table: SV39PageTable<FrameAllocator>,
    segments: Vec<Segment>,
}

impl Space for UserSpace {
    fn new_bare() -> Self {
        Self {
            page_table: SV39PageTable::<FrameAllocator>::new(),
            segments: Vec::new(),
        }
    }

    fn page_table(&mut self) -> &mut SV39PageTable<FrameAllocator> {
        &mut self.page_table
    }
}

impl UserSpace {
    pub fn new(elf_data: &[u8]) -> (Self, usize) {
        let mut memory_space = Self::new_bare();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        let trampoline = Segment::new_identical(strampoline as usize..etrampoline as usize);
        memory_space.map(trampoline.clone());
        memory_space.segments.push(trampoline);

        let elf = ElfFile::read(elf_data);
        let program_segments = elf.programs();
        for program_segment in &program_segments {
            let program =
                Segment::new_user_accessible(program_segment.start_va()..program_segment.end_va());
            memory_space.map(program.clone());
            memory_space.segments.push(program);
            memory_space.page_table.write(
                program_segment.start_va(),
                program_segment.memory_size(),
                program_segment.data,
            );
        }

        let stack = Segment::new_user_accessible(USER_STACK_START_ADDR..USER_STACK_END_ADDR);
        memory_space.map(stack.clone());
        memory_space.segments.push(stack);

        let heap = Segment::new_user_accessible(USER_HEAP_START_ADDR..USER_HEAP_END_ADDR);
        memory_space.map(heap.clone());
        memory_space.segments.push(heap);

        (memory_space, elf.entry_address())
    }
}

impl Clone for UserSpace {
    fn clone(&self) -> Self {
        let mut memory_space = Self::new_bare();
        for segment in &self.segments {
            memory_space.map(segment.clone());
            let va = segment.va_range.start;
            let len = segment.va_range.len();
            let src = self.page_table.translate_buffer(va, len);
            let mut dst = memory_space.page_table.translate_buffer(va, len);
            dst.copy_from_slice(&src);
        }
        memory_space
    }
}

pub struct KernelSpace {
    pub page_table: SV39PageTable<FrameAllocator>,
}

impl Space for KernelSpace {
    fn new_bare() -> Self {
        Self {
            page_table: SV39PageTable::<FrameAllocator>::new(),
        }
    }

    fn page_table(&mut self) -> &mut SV39PageTable<FrameAllocator> {
        &mut self.page_table
    }
}

impl KernelSpace {
    pub fn new() -> Self {
        let mut memory_space = Self::new_bare();

        memory_space.map(Segment::new_identical(skernel as usize..ekernel as usize));
        memory_space.map(Segment::new_identical(
            ekernel as usize..AVAILABLE_MEMORY_END,
        ));
        memory_space.map(Segment::new_identical(0x10007000..0x10008000));
        memory_space.map(Segment::new_identical(0x10008000..0x10009000)); // MMIO VIRT_TEST/RTC  in virt machine
        memory_space.map(Segment::new_identical(DMA_START_ADDR..DMA_END_ADDR));

        // 内核栈
        memory_space.map(Segment::new_identical(
            KERNEL_STACK_START_ADDR..KERNEL_STACK_END_ADDR,
        ));
        memory_space.map(Segment::new_identical(
            KERNEL_HEAP_START_ADDR..KERNEL_HEAP_END_ADDR,
        ));
        memory_space
    }
}

static_var! {
    KERNEL_SPACE: KernelSpace = KernelSpace::new();
}

#[derive(Clone)]
pub struct Segment {
    pub va_range: Range<usize>,
    pub identical: bool,
    pub user_accessible: bool,
}

impl Segment {
    fn new_identical(va_range: Range<usize>) -> Self {
        Self {
            va_range,
            identical: true,
            user_accessible: false,
        }
    }

    fn new_user_accessible(va_range: Range<usize>) -> Self {
        Self {
            va_range,
            identical: false,
            user_accessible: true,
        }
    }

    pub fn vpn_range(&self) -> Range<usize> {
        let start_vpn = VA::new(self.va_range.start)
            .align_to_lower()
            .page()
            .number();
        let end_vpn = VA::new(self.va_range.end).align_to_upper().page().number();
        start_vpn..end_vpn
    }
}
