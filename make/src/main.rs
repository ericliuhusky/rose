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

struct Makefile {
    link_arg: &'static str,
    nightly: bool,
    dir: &'static str,
    users: Option<[User; 3]>,
}

struct User {
    bin: &'static str,
    enrty: Option<usize>,
}

const CH0: Makefile = Makefile {
    link_arg: "-Ttext=0x80200000",
    nightly: false,
    dir: "../ch0",
    users: None,
};

const CH1: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch1",
    users: None,
};

const CH2: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch2",
    users: Some([
        User {
            bin: "hello_world",
            enrty: Some(0x80400000),
        },
        User {
            bin: "priv_inst",
            enrty: Some(0x80400000),
        },
        User {
            bin: "store_fault",
            enrty: Some(0x80400000),
        },
    ]),
};

const CH3: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch3",
    users: Some([
        User {
            bin: "00write_a",
            enrty: Some(0x80600000),
        },
        User {
            bin: "01write_b",
            enrty: Some(0x80620000),
        },
        User {
            bin: "02write_c",
            enrty: Some(0x80640000),
        },
    ]),
};

const CH4: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch4",
    users: Some([
        User {
            bin: "00write_a",
            enrty: None,
        },
        User {
            bin: "01write_b",
            enrty: None,
        },
        User {
            bin: "02write_c",
            enrty: None,
        },
    ]),
};

const CH5: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch5",
    users: Some([
        User {
            bin: "initproc",
            enrty: None,
        },
        User {
            bin: "shell",
            enrty: None,
        },
        User {
            bin: "fork",
            enrty: None,
        },
    ]),
};

fn main() {
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    fn rustflags(link_arg: &str) -> String {
        format!(
            r#"target.{}.rustflags = ["-Clink-arg={}"]"#,
            TARGET, link_arg
        )
    }

    for ch in [CH0, CH1, CH2, CH3, CH4, CH5] {
        let mut makefile = String::from("run:\n");
        if let Some(users) = ch.users {
            let mut build_user = String::from("cd ../user");
            build_user.push_str(format!(" && {}", clean()).as_str());
            for user in users {
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
            makefile.push_str(format!("\t@{}\n", build_user).as_str());
        }

        let config = rustflags(ch.link_arg);
        let build_cmd = build(ch.nightly, Some(&config), None);

        let elf_to_bin_cmd = elf_to_bin(&kernel_elf, &kernel_bin);
        let qemu_cmd = qemu_run(&kernel_bin);

        makefile.push_str(
            format!(
                "\t@{}\n\t@{}\n\t@{}\n\t@{}\n",
                clean(),
                build_cmd,
                elf_to_bin_cmd,
                qemu_cmd
            )
            .as_str(),
        );

        let mut f = File::create(format!("{}/Makefile", ch.dir).as_str()).unwrap();
        f.write_all(makefile.as_bytes()).unwrap();
    }
}
