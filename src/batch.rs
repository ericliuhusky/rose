//! batch subsystem

use core::arch::asm;
use crate::csr::{sepc, sscratch};

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("[kernel] All applications completed!");

            crate::exit::exit();
        }
        println!("[kernel] Loading app_{}", app_id);
        // clear app area
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
    }
}


static mut APP_MANAGER: AppManager = AppManager {num_app:0, current_app:0, app_start:[0; MAX_APP_NUM + 1]};

/// init batch subsystem
pub fn init() {
    unsafe {
        extern "C" {
            fn _num_app();
        }
        let num_app_ptr = _num_app as usize as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
        let app_start_raw: &[usize] =
            core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
        app_start[..=num_app].copy_from_slice(app_start_raw);
        APP_MANAGER = AppManager {
            num_app,
            current_app: 0,
            app_start,
        };

        APP_MANAGER.print_app_info();
    }
}


/// run next app
pub fn run_next_app() {
    unsafe {
        let current_app = APP_MANAGER.current_app;
        APP_MANAGER.load_app(current_app);
        APP_MANAGER.current_app += 1;

        sepc::write(APP_BASE_ADDRESS);
        sscratch::write(KERNEL_STACK.get_sp());
        asm!("mv sp, {}", in(reg) USER_STACK.get_sp());
        asm!("sret");
    }
}
