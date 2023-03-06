use super::config::{LinkArg, Makefile, BOOTLOADER, KERNEL_ENTRY, TARGET};
use std::{fs::File, io::Write};

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

pub fn create_makefile(ch: &Makefile) {
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
        \t@{0}\n\
        \t@cargo clean\n\
        \t@{1}\n\
        \t@rust-objcopy target/{2}/release/kernel --strip-all -O binary target/{2}/release/kernel.bin\n\
        \t@qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios {3} \
            -device loader,file=target/{2}/release/kernel.bin,addr={4}",
        build_user,
        build(ch.nightly, Some(&ch.link_arg), None),
        TARGET,
        BOOTLOADER,
        KERNEL_ENTRY
    )
    .unwrap();
}
