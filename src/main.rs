#![no_std]
#![no_main]

mod puts;
mod exit;
mod rust_no_std;

#[no_mangle]
fn _start() {
    puts::puts("Hello, world!\n");

    exit::exit();
}
