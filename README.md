1. Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. `rustup target add riscv64gc-unknown-none-elf`
3. cargo-binutils `cargo install cargo-binutils & rustup component add llvm-tools-preview`
4. qemu-riscv64 `brew install qemu`
