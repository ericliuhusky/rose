mod config;

use config::ch;
use std::{fs::File, io::Write};

const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

fn clean() -> String {
    String::from("cargo clean")
}

fn build(nightly: bool, config: Option<&str>, bin: Option<&str>) -> String {
    let mut cmd = String::from("cargo");
    if nightly {
        cmd.push_str(" +nightly");
    }
    cmd.push_str(" build");
    if let Some(config) = config {
        cmd.push_str(format!(" --config '{}'", config).as_str());
    }
    cmd.push_str(format!(" --target {}", TARGET).as_str());
    if let Some(bin) = bin {
        cmd.push_str(format!(" --bin {}", bin).as_str());
    }
    cmd.push_str(" --release");
    cmd
}

fn elf_to_bin(kernel_elf: &str, kernel_bin: &str) -> String {
    format!(
        "rust-objcopy {} --strip-all -O binary {}",
        kernel_elf, kernel_bin
    )
}

fn qemu_run(kernel_bin: &str) -> String {
    format!(
        "qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios {} \
            -device loader,file={},addr={}",
        BOOTLOADER, kernel_bin, KERNEL_ENTRY
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
        let mut build_user = String::new();
        if !ch.users.is_empty() {
            build_user.push_str("cd ../user");
            build_user.push_str(format!(" && {}", clean()).as_str());
            for user in &ch.users {
                let build_cmd;
                if let Some(entry) = user.enrty {
                    let link_arg = format!("-Ttext={:x}", entry);
                    let config = rustflags(&link_arg);
                    build_cmd = build(true, Some(&config), Some(user.bin));
                } else {
                    build_cmd = build(true, None, Some(user.bin));
                }
                build_user.push_str(format!(" && {}", build_cmd).as_str())
            }
        }

        let config = rustflags(ch.link_arg);

        let mut f = File::create(format!("{}/Makefile", ch.dir).as_str()).unwrap();
        writeln!(
            f,
            "run:\n\t@{}\n\t@{}\n\t@{}\n\t@{}\n\t@{}\n",
            build_user,
            clean(),
            build(ch.nightly, Some(&config), None),
            elf_to_bin(&kernel_elf, &kernel_bin),
            qemu_run(&kernel_bin)
        )
        .unwrap();
    }
}
