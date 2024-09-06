TARGET := riscv64imac-unknown-none-elf
KERNEL_ELF := kernel/target/$(TARGET)/release/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_ENTRY_ADDR := 0x80200000
BOOTLOADER_ELF := sbi/target/$(TARGET)/release/sbi
BOOTLOADER_BIN := $(BOOTLOADER_ELF).bin
FS_IMG := user/target/$(TARGET)/release/fs.img
TCP_PORT_80_MAP := 3000
UDP_PORT_2000_MAP := 6000


run:
	@cd sbi && cargo build --release --target $(TARGET)
	@rust-objcopy $(BOOTLOADER_ELF) --strip-all -O binary $(BOOTLOADER_BIN)

	@cd kernel && cargo build --release
	@rust-objcopy $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

	@cd user && cargo build --release
	@cd fs_pack && cargo run
	
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER_BIN) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_ADDR) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
    	-device virtio-blk-device,drive=x0 \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=udp::$(UDP_PORT_2000_MAP)-:2000,hostfwd=tcp::$(TCP_PORT_80_MAP)-:80	

clean:
	@cd kernel && cargo clean
	@cd user && cargo clean
	@cd sbi && cargo clean
