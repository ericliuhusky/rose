use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;
use elf_reader::ElfFile;
use exception::context::Context;
use frame_allocator::FrameAllocator;
use lazy_static::lazy_static;

pub const MEMORY_END: usize = 0x88000000;

pub const KERNEL_STACK_START_ADDR: usize = HIGH_START_ADDR;
pub const KERNEL_STACK_END_ADDR: usize = KERNEL_STACK_START_ADDR + 0x2000;
pub const KERNEL_STACK_TOP: usize = KERNEL_STACK_END_ADDR;

extern "C" {
    fn skernel();
    fn ekernel();
    fn strampoline();
    fn etrampoline();
}

use page_table::{SV39PageTable, HIGH_START_ADDR};
use page_table::{VA, VPN};

pub struct MemorySpace {
    pub page_table: SV39PageTable<FrameAllocator>,
    segments: Vec<Segment>,
}

impl MemorySpace {
    fn map(&mut self, segment: Segment) {
        for vpn in segment.vpn_range() {
            self.page_table
                .map(vpn, segment.identical, segment.user_accessible);
        }
        self.segments.push(segment);
    }

    fn new_bare() -> Self {
        Self {
            page_table: SV39PageTable::<FrameAllocator>::new(),
            segments: Vec::new(),
        }
    }

    pub fn new_kernel() -> Self {
        let mut memory_space = Self::new_bare();

        memory_space.map(Segment::new_identical(skernel as usize..ekernel as usize));
        memory_space.map(Segment::new_identical(ekernel as usize..MEMORY_END));
        memory_space.map(Segment::new_identical(0x100000..0x102000)); // MMIO VIRT_TEST/RTC  in virt machine
        memory_space.map(Segment::new_identical(0x10001000..0x10002000)); // MMIO VIRT_TEST/RTC  in virt machine

        // 内核栈
        memory_space.map(Segment::new(KERNEL_STACK_START_ADDR..KERNEL_STACK_END_ADDR));
        memory_space
    }

    pub fn new_user(elf_data: &[u8]) -> (Self, usize) {
        let mut memory_space = Self::new_bare();

        // 将__trap_entry映射到用户地址空间，并使之与内核地址空间的地址相同
        memory_space.map(Segment::new_identical(
            strampoline as usize..etrampoline as usize,
        ));

        let elf = ElfFile::read(elf_data);
        let program_segments = elf.programs();
        for program_segment in &program_segments {
            memory_space.map(Segment::new_user_accessible(
                program_segment.start_va()..program_segment.end_va(),
            ));
            memory_space.page_table.write(
                program_segment.start_va(),
                program_segment.memory_size(),
                program_segment.data,
            );
        }

        memory_space.map(Segment::new_user_accessible(
            0xFFFFFFFFFFFCF000..0xFFFFFFFFFFFEF000,
        ));

        (memory_space, elf.entry_address())
    }
}

impl Clone for MemorySpace {
    fn clone(&self) -> Self {
        let mut memory_space = Self::new_bare();
        for segment in &self.segments {
            memory_space.map(segment.clone());
            let va = segment.va_range.start;
            let len = segment.va_range.len();
            let src_bufs = self.page_table.translate_buffer(va, len);
            let mut dst_bufs = memory_space.page_table.translate_buffer(va, len);
            for i in 0..src_bufs.len() {
                for j in 0..src_bufs[i].len() {
                    dst_bufs[i][j] = src_bufs[i][j];
                }
            }
        }
        memory_space
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: MemorySpace = MemorySpace::new_kernel();
}

#[derive(Clone)]
pub struct Segment {
    pub va_range: Range<usize>,
    pub identical: bool,
    pub user_accessible: bool,
}

impl Segment {
    fn new(va_range: Range<usize>) -> Self {
        Self {
            va_range,
            identical: false,
            user_accessible: false,
        }
    }

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
            .page_number()
            .0;
        let end_vpn = VA::new(self.va_range.end).align_to_upper().page_number().0;
        start_vpn..end_vpn
    }
}
