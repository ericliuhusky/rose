TARGET_DIR := target/riscv64gc-unknown-none-elf/release/

build:
	@cargo clean
	@python3 build.py
	@rust-objcopy $(TARGET_DIR)/00write_a  --strip-all -O binary $(TARGET_DIR)/00write_a.bin
	@rust-objcopy $(TARGET_DIR)/01write_b  --strip-all -O binary $(TARGET_DIR)/01write_b.bin
	@rust-objcopy $(TARGET_DIR)/02write_c  --strip-all -O binary $(TARGET_DIR)/02write_c.bin
