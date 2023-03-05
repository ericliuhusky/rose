target := riscv64gc-unknown-none-elf
link-arg := -Tsrc/linker.ld
config := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg)"]'

link-arg-user1 := -Ttext=0x80400000
config-user1 := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg-user1)"]'
link-arg-user2 := -Ttext=0x80420000
config-user2 := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg-user2)"]'
link-arg-user3 := -Ttext=0x80440000
config-user3 := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg-user3)"]'

内核ELF文件 := target/$(target)/release/rcore
内核二进制文件 := $(内核ELF文件).bin
引导程序 := ../rustsbi-qemu.bin
内核入口地址 := 0x80200000

all: run

build_user:
	@cd ../user && \
		cargo clean && \
		cargo build --config $(config-user1) --target $(target) --bin 00write_a --release && \
		cargo build --config $(config-user2) --target $(target) --bin 01write_b --release && \
		cargo build --config $(config-user3) --target $(target) --bin 02write_c --release


build: build_user
	@cargo clean
	@cargo build --config $(config) --target $(target) --release
	@rust-objcopy $(内核ELF文件) --strip-all -O binary $(内核二进制文件)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(引导程序) \
		-device loader,file=$(内核二进制文件),addr=$(内核入口地址)
