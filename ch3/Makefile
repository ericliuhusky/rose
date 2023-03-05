内核ELF文件 := target/riscv64gc-unknown-none-elf/release/rcore
内核二进制文件 := $(内核ELF文件).bin
引导程序 := ../rustsbi-qemu.bin
内核入口地址 := 0x80200000

all: run

build_user:
	@cd ../user && \
		cargo clean && \
		cargo build --config 'target.riscv64gc-unknown-none-elf.rustflags = ["-Clink-arg=-Ttext=0x80400000"]' --target riscv64gc-unknown-none-elf --bin 00write_a --release && \
		cargo build --config 'target.riscv64gc-unknown-none-elf.rustflags = ["-Clink-arg=-Ttext=0x80420000"]' --target riscv64gc-unknown-none-elf --bin 01write_b --release && \
		cargo build --config 'target.riscv64gc-unknown-none-elf.rustflags = ["-Clink-arg=-Ttext=0x80440000"]' --target riscv64gc-unknown-none-elf --bin 02write_c --release


build: build_user
	@cargo clean
	@cargo build --config 'target.riscv64gc-unknown-none-elf.rustflags = ["-Clink-arg=-Tsrc/linker.ld"]' --target riscv64gc-unknown-none-elf --release
	@rust-objcopy $(内核ELF文件) --strip-all -O binary $(内核二进制文件)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(引导程序) \
		-device loader,file=$(内核二进制文件),addr=$(内核入口地址)
