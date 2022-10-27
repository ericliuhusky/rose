FROM rust

RUN rustup target add riscv64gc-unknown-none-elf; \
    cargo install cargo-binutils; \
    rustup component add llvm-tools-preview

RUN apt update; \
    apt install -y qemu-system-riscv64
