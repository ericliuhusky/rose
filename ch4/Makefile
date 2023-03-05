target := riscv64gc-unknown-none-elf
link-arg := -Tsrc/linker.ld
config := 'target.$(target).rustflags = ["-Clink-arg=$(link-arg)"]'

KERNEL_ELF := target/$(target)/release/rcore
KERNEL_BIN := $(KERNEL_ELF).bin
BOOTLOADER := ../rustsbi-qemu.bin
KERNEL_ENTRY_PA := 0x80200000

all: run

build_user:
	@cd ../user \
		&& cargo clean \
		&& cargo build --release

build: build_user
	@cargo clean
	@cargo build --config $(config) --target $(target) --release
	@rust-objcopy $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
