mod config;

use config::{ch, LinkArg, Makefile};
use std::{fs::File, io::Write};

const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

fn build(nightly: bool, link_arg: Option<&LinkArg>, bin: Option<&str>) -> String {
    let nightly = if nightly { " +nightly" } else { "" };
    let config = if let Some(link_arg) = link_arg {
        format!(
            " --config 'target.{}.rustflags = [\"-Clink-arg={}\"]'",
            TARGET,
            match link_arg {
                LinkArg::Address(arg) => format!("-Ttext={:x}", arg),
                LinkArg::File(arg) => format!("-T{}", arg),
            }
        )
    } else {
        String::new()
    };
    let bin = if let Some(bin) = bin {
        format!(" --bin {}", bin)
    } else {
        String::new()
    };
    format!(
        "cargo{} build{} --target {}{} --release",
        nightly, config, TARGET, bin
    )
}

fn create_makefile(ch: &Makefile, kernel_elf: &str, kernel_bin: &str) {
    let build_user = if ch.users.is_empty() {
        String::new()
    } else {
        let build_user_bins: String = ch
            .users
            .iter()
            .map(|user| {
                format!(
                    "&& {} ",
                    build(true, user.link_arg.as_ref(), Some(user.bin))
                )
            })
            .collect();

        format!(
            "cd ../user \
            && cargo clean \
            {}",
            build_user_bins
        )
    };

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
        build(ch.nightly, Some(&ch.link_arg), None),
        kernel_elf,
        kernel_bin,
        BOOTLOADER,
        kernel_bin,
        KERNEL_ENTRY
    )
    .unwrap();
}

fn main() {
    config::init();
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    for ch in ch() {
        create_makefile(ch, &kernel_elf, &kernel_bin);
    }
}
