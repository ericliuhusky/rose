pub fn 读取异常() -> 异常 {
    let 触发异常原因代码: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) 触发异常原因代码);
    }
    if 触发异常原因代码 >> 63 == 1 {
        异常::中断(中断::解析(触发异常原因代码 & 0xf))
    } else {
        异常::解析(触发异常原因代码)
    }
}

pub enum 异常 {
    用户系统调用,
    存储错误,
    存储页错误,
    非法指令,
    中断(中断),
    其它
}

impl 异常 {
    fn 解析(触发异常原因代码: usize) -> Self {
        match 触发异常原因代码 {
            2 => 异常::非法指令,
            7 => 异常::存储错误,
            15 => 异常::存储页错误,
            8 => 异常::用户系统调用,
            _ => 异常::其它
        }
    }
}

pub enum 中断 {
    时钟中断,
    其它
}

impl 中断 {
    pub fn 解析(触发异常原因代码: usize) -> Self {
        match 触发异常原因代码 {
            5 => 中断::时钟中断,
            _ => 中断::其它
        }
    }
}
