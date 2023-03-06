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

fn create_link_app(ch: &Makefile) {
    if ch.users.is_empty() {
        return;
    }
    let mut f = File::create(format!("{}/src/link_app.s", ch.dir)).unwrap();
    writeln!(
        f,
        r#"# created by crate make
    .section .data
    .globl _num_app
_num_app:
    .quad {}"#,
        ch.users.len()
    )
    .unwrap();
    for i in 0..ch.users.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i).unwrap();
    }
    writeln!(f, r#"    .quad app_{}_end"#, ch.users.len() - 1).unwrap();
    writeln!(
        f,
        r#"
    .globl _app_names
_app_names:"#
    )
    .unwrap();
    for user in &ch.users {
        writeln!(f, r#"    .string "{}""#, user.bin).unwrap();
    }
    for (i, user) in ch.users.iter().enumerate() {
        writeln!(
            f,
            r#"
    .section .data
    .globl app_{0}_start
    .globl app_{0}_end
app_{0}_start:
    .incbin "../user/target/{1}/release/{2}"
app_{0}_end:"#,
            i, TARGET, user.bin
        )
        .unwrap();
    }
}

fn main() {
    config::init();
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    for ch in ch() {
        create_makefile(ch, &kernel_elf, &kernel_bin);
        create_link_app(ch);
    }
}
