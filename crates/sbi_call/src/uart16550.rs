use core::cell::UnsafeCell;

const UART_BASE: usize = 0x10000000;

static mut UART: *const Uart16550 = UART_BASE as *const Uart16550;

#[repr(C)]
pub struct Uart16550 {
    rbr_thr: UnsafeCell<u8>,
    _unused: [u8; 4],
    lsr: UnsafeCell<u8>,
}

impl Uart16550 {
    fn line_status(&self) -> u8 {
        unsafe { self.lsr.get().read_volatile() }
    }

    fn is_data_ready(&self) -> bool {
        self.line_status() & 1 == 1
    }

    fn is_transmitter_fifo_empty(&self) -> bool {
        (self.line_status() >> 5) & 1 == 1
    }
}

pub fn getchar() -> u8 {
    unsafe {
        let uart = &(*UART);
        loop {
            if uart.is_data_ready() {
                break;
            }
        }
        uart.rbr_thr.get().read_volatile()
    }
}

pub fn putchar(c: u8) {
    unsafe {
        let uart = &(*UART);
        assert_eq!(uart.is_transmitter_fifo_empty(), true);
        uart.rbr_thr.get().write_volatile(c);
    }
}
