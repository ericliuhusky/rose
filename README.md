# Rust Operating System Elegant

## QuickStart

1. Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. `rustup target add riscv64imac-unknown-none-elf`
3. cargo-binutils `cargo install cargo-binutils & rustup component add llvm-tools-preview`
4. qemu-riscv64 `brew install qemu`
5. make

## Todo-List

- 代码内的TODO
- Fix waring
- 注释、测试、日志

### kernel

1. 第七章 命令行参数与标准I/O重定向, 信号
2. 第八章 条件变量机制
3. rustsbi-qemu引导程序
4. 串口驱动 -serial stdio
5. 中断阻塞代替忙等
6. 除了保存用户上下文外，还应保存内核上下文，以保存内核的执行状态，以便能在恢复执行的时候恢复到内核某一位置继续执行，而不是只能恢复到用户ecall的下一条指令
7. GUI图形交互

### net

1. 分组转发给各个端口
2. tcp, udp的客户端

## Developer Log

1. 保存和恢复上下文的汇编可以分为三部分：地址空间的切换，特殊寄存器的保存和恢复，通用寄存器的保存和恢复。
其中只有通用寄存器的保存和恢复必须使用汇编。希望保留最少的汇编代码，因此决定抽出来一个rust函数，我们把它
叫做rust_restore，汇编的部分叫做_restore。开启地址空间时，rust_restore必须声明`#[inline(never)]`。保存和恢复上下文位于内核地址空间和用户地址空间都可见的跳板处，内核恢复用户程序时，程序计数器由内核走到位于跳板的rust_restore，进行地址空间的切换，接着由rust_restore走到位于跳板的_restore。一旦rust_restore被内联到内核，内核恢复用户程序时，程序计数器还指向内核代码，进行地址空间切换，用户地址空间里程序计数器无法被翻译到正确的地址，就无法跳到_restore。切换完地址空间后，还需要读写特殊寄存器，为了方便封装了一个库，这个读写函数则必须被声明为`#[inline(always)]`。如果这个函数不被内联，那么在切换完地址空间之后，还要去调位于内核地址空间的读写函数代码，显然是跳转不过去的；而内联直接替换为特殊寄存器读写的汇编代码显然就没有问题。

2. 内存对齐的问题，汇编代码要声明.align 3，rust裸函数要声明#[repr(align(8))]，遇到了因没有内存对齐导致设置stvec失败的问题
