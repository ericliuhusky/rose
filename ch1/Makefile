target := riscv64gc-unknown-none-elf
link-arg := -Tsrc/linker.ld
config := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg)"]'

内核ELF文件 := target/$(target)/release/kernel
内核二进制文件 := $(内核ELF文件).bin
引导程序 := ../rustsbi-qemu.bin
内核入口地址 := 0x80200000

all: run

build:
	@cargo clean
	@cargo +nightly build --config $(config) --target $(target) --release
	@rust-objcopy $(内核ELF文件) --strip-all -O binary $(内核二进制文件)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(引导程序) \
		-device loader,file=$(内核二进制文件),addr=$(内核入口地址)
