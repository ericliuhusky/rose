TARGET_DIR := target/riscv64gc-unknown-none-elf/release/

build:
	@cargo clean
	@cargo build --release
	@rust-objcopy $(TARGET_DIR)hello_world  --strip-all -O binary $(TARGET_DIR)hello_world.bin
	@rust-objcopy $(TARGET_DIR)store_fault  --strip-all -O binary $(TARGET_DIR)store_fault.bin
	@rust-objcopy $(TARGET_DIR)priv_inst  --strip-all -O binary $(TARGET_DIR)priv_inst.bin
