TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF := target/$(TARGET)/release/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_ENTRY_ADDR := 0x80200000
BOOTLOADER := ../rustsbi-qemu.bin
FS_IMG := ../user/target/$(TARGET)/release/fs.img
TCP_PORT_80_MAP := 3000
UDP_PORT_2000_MAP := 6000


run: kernel-bin fs-img
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_ADDR) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
    	-device virtio-blk-device,drive=x0 \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=udp::$(UDP_PORT_2000_MAP)-:2000,hostfwd=tcp::$(TCP_PORT_80_MAP)-:80

build:
	@cargo build --release

kernel-bin: build
	@rust-objcopy $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

build-user:
	@cd ../user && cargo build --release

fs-img: build-user
	@cd ../fs_pack && cargo run
