use riscv::register::{time, sie};
use sbi_call::set_timer;

const 一毫秒时钟计数器的增量: usize = 12500;

fn 读取时钟计数器的值() -> usize {
    time::read()
}

pub fn 读取时钟计数器的毫秒值() -> usize {
    读取时钟计数器的值() / 一毫秒时钟计数器的增量
}

pub fn 为下一次时钟中断定时() {
    // 10ms后触发时钟中断
    set_timer(读取时钟计数器的值() + 一毫秒时钟计数器的增量 * 10);
}

pub fn 开启时钟中断() {
    unsafe {
        sie::set_stimer();
    }
}
