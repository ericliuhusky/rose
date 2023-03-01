use core::arch::asm;

const 一毫秒时钟计数器的增量: usize = 12500;

fn 读取时钟计数器的值() -> usize {
    let 时钟计数器的值: usize;
    unsafe {
        asm!("csrr {}, time", out(reg) 时钟计数器的值);
    }
    时钟计数器的值
}

pub fn 读取时钟计数器的毫秒值() -> usize {
    读取时钟计数器的值() / 一毫秒时钟计数器的增量
}

fn 设置触发时钟中断的时钟计数器的值(时钟计数器的值: usize) {
    unsafe {
        asm!(
            "ecall",
            in("x10") 时钟计数器的值,
            in("x17") 0,
        );
    }
}

pub fn 为下一次时钟中断定时() {
    // 10ms后触发时钟中断
    设置触发时钟中断的时钟计数器的值(读取时钟计数器的值() + 一毫秒时钟计数器的增量 * 10);
}

pub fn 开启时钟中断() {
    unsafe {
        asm!("csrw sie, {}", in(reg) 1 << 5);
    }
}
