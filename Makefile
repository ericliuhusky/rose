KERNEL_ELF := target/riscv64gc-unknown-none-elf/release/rcore
KERNEL_BIN := $(KERNEL_ELF).bin
BOOTLOADER := ./rustsbi-qemu.bin
KERNEL_ENTRY_PA := 0x80200000

all: run

build:
	@cargo clean
	@cargo build --release
	@rust-objcopy $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
