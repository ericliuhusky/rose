#![no_std]

/// cause register 触发异常的原因
pub mod scause;
/// trap vector base address register 异常控制流入口地址
pub mod stvec;
/// time register 时钟计数器
pub mod time;
/// interrupt enable register 中断开启寄存器
pub mod sie;
/// address translation and protection register 地址翻译寄存器，存放分页方式(SV39)和页表根页号
pub mod satp;
/// exception program counter 异常程序计数器，记录触发异常的地址
pub mod sepc;
/// 临时记录寄存器
pub mod sscratch;
