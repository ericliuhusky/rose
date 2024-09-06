#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![deny(warnings)]

#[macro_use]
extern crate core_ext;
mod trap;
mod uart16550;

use core::arch::asm;
use core_ext::CoreExt;
use riscv::register::{mcause::Interrupt, medeleg, mepc, mideleg, mie, mip, mstatus, mtvec, time};
use trap::{trap_entry, CONTEXT};
use uart16550::{getchar, putchar};

const SUPERVISOR_ENTRY: usize = 0x8020_0000;
const TEST_PASS: u32 = 0x5555;
const TEST_BASE: usize = 0x100000;
const MTIMER_BASE: usize = 0x2004000;

fn shutdown() -> ! {
    unsafe {
        *(TEST_BASE as *mut u32) = TEST_PASS;
    }
    unreachable!()
}

fn set_timer(time_value: usize) {
    unsafe {
        *(MTIMER_BASE as *mut usize) = time_value;
    }
}

/// 入口。
///
/// # Safety
///
/// 裸函数。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    asm!(
        "   li sp, {SUPERVISOR_ENTRY}
            call {rust_main}
        ",
        SUPERVISOR_ENTRY = const SUPERVISOR_ENTRY,
        rust_main    = sym rust_main,
        options(noreturn),
    )
}

struct CoreExtImpl;

impl CoreExt for CoreExtImpl {
    fn putchar(&self, c: char) {
        putchar(c as u8);
    }

    fn exit(&self) -> ! {
        shutdown()
    }
}

/// rust 入口。
fn rust_main() {
    core_ext::init(&CoreExtImpl);
    println!("[sbi] Hello!");
    // 设置并打印 pmp
    set_pmp();
    set_timer(usize::MAX);
    // 准备启动调度
    unsafe {
        mideleg::set_stimer();
        medeleg::set_user_env_call();
        medeleg::set_illegal_instruction();
        medeleg::set_store_page_fault();
        mtvec::write(trap_entry as _, mtvec::TrapMode::Direct);

        // 开启时钟中断
        mie::set_mtimer();
        // MPP 字段设置为 Supervisor 模式 (01)
        mstatus::set_mpp(mstatus::MPP::Supervisor);

        mepc::write(SUPERVISOR_ENTRY);
        asm!("mret");
    }
}

/// 设置 PMP。
fn set_pmp() {
    use riscv::register::*;
    unsafe {
        pmpcfg0::set_pmp(0, Range::OFF, Permission::NONE, false);
        pmpaddr0::write(0);
        pmpcfg0::set_pmp(1, Range::TOR, Permission::RWX, false);
        pmpaddr1::write(0x88000000 >> 2);
    }
}

fn trap_handler() {
    use riscv::register::mcause::{self, Exception as E, Trap as T};
    match mcause::read().cause() {
        T::Exception(E::SupervisorEnvCall) => {
            let id = unsafe { CONTEXT.x[17] };
            let a0 = unsafe { CONTEXT.x[10] };
            match id {
                0 => {
                    unsafe {
                        mip::clear_stimer();
                    }
                    set_timer(a0);
                }
                1 => unsafe {
                    CONTEXT.x[10] = time::read();
                },
                2 => shutdown(),
                3 => {
                    putchar(a0 as u8);
                }
                4 => unsafe {
                    CONTEXT.x[10] = getchar() as usize;
                },
                _ => {}
            }

            let pc = mepc::read();
            mepc::write(pc + 4);
        }
        T::Interrupt(Interrupt::MachineTimer) => {
            set_timer(usize::MAX);

            unsafe {
                // 设置stip，触发S模式时钟中断
                mip::set_stimer();
            }
        }
        _ => {}
    }
}
