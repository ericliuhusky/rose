pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const MEMORY_END: usize = 0x80800000;

pub const TRAP_CONTEXT_END: usize = 0xfffffffffffff000;
pub const TRAP_CONTEXT: usize = 0xffffffffffffe000;
pub const 内核栈栈顶: usize = 0xfffffffffffff000;
pub const 内核栈栈底: usize = 0xffffffffffffd000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];
