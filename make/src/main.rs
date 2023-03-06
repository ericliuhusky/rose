mod config;

use config::ch;
use std::{fs::File, io::Write};

const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

fn build(nightly: bool, config: Option<&str>, bin: Option<&str>) -> String {
    let nightly = if nightly { " +nightly" } else { "" };
    let config = if let Some(config) = config {
        format!("--config '{}'", config)
    } else {
        String::new()
    };
    let bin = if let Some(bin) = bin {
        format!("--bin {}", bin)
    } else {
        String::new()
    };
    format!(
        "cargo{} build {} --target {} {} --release",
        nightly, config, TARGET, bin
    )
}

fn main() {
    config::init();
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    fn rustflags(link_arg: &str) -> String {
        format!(
            r#"target.{}.rustflags = ["-Clink-arg={}"]"#,
            TARGET, link_arg
        )
    }

    for ch in ch() {
        let build_user = if ch.users.is_empty() {
            String::new()
        } else {
            let mut users = String::new();
            for user in &ch.users {
                let build_cmd = if let Some(entry) = user.enrty {
                    let link_arg = format!("-Ttext={:x}", entry);
                    let config = rustflags(&link_arg);
                    build(true, Some(&config), Some(user.bin))
                } else {
                    build(true, None, Some(user.bin))
                };
                users.push_str(format!(" && {}", build_cmd).as_str());
            }
            format!("cd ../user && cargo clean {}", users)
        };

        let config = rustflags(ch.link_arg);

        let mut f = File::create(format!("{}/Makefile", ch.dir).as_str()).unwrap();
        writeln!(
            f,
            "run:\n\
            \t@{}\n\
            \t@cargo clean\n\
            \t@{}\n\
            \t@rust-objcopy {} --strip-all -O binary {}\n\
            \t@qemu-system-riscv64 \
                -machine virt \
                -nographic \
                -bios {} \
                -device loader,file={},addr={}",
            build_user,
            build(ch.nightly, Some(&config), None),
            kernel_elf,
            kernel_bin,
            BOOTLOADER,
            kernel_bin,
            KERNEL_ENTRY
        )
        .unwrap();
    }
}
