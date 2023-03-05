const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

use std::process::Command;

fn clean(dir: &str) {
    Command::new("cargo")
        .current_dir(dir)
        .arg("clean")
        .spawn()
        .expect("clean")
        .wait()
        .expect("wait clean");
}

fn build(dir: &str, config: Option<&str>, nightly: bool, bin: Option<&str>) {
    let mut cmd = Command::new("cargo");
    if nightly {
        cmd.arg("+nightly");
    }
    if let Some(config) = config {
        cmd.args(["--config", config]);
    }
    cmd
        .current_dir(dir) 
        .arg("build")
        .args(["--target", TARGET]);
    if let Some(bin) = bin {
        cmd.args(["--bin", bin]);
    }
    cmd
        .arg("--release")
        .spawn()
        .expect("build")
        .wait()
        .expect("wait build");
}

fn elf_to_bin(dir: &str, kernel_elf: &str, kernel_bin: &str) {
    Command::new("rust-objcopy")
        .current_dir(dir)
        .arg(kernel_elf)
        .arg("--strip-all")
        .args(["-O", "binary", kernel_bin])
        .spawn()
        .expect("elf_to_bin")
        .wait()
        .expect("wait elf_to_bin");
}

fn qemu_run(dir: &str, kernel_bin: &str) -> String {
    Command::new("qemu-system-riscv64")
        .current_dir(dir)
        .args(["-machine", "virt"])
        .arg("-nographic")
        .args(["-bios", BOOTLOADER])
        .args(["-device", &format!("loader,file={},addr={}", kernel_bin, KERNEL_ENTRY)])
        .output()
        .expect("qemu_run")
        .stdout
        .iter().map(|b| *b as char).collect()
}

struct Makefile {
    link_arg: &'static str,
    nightly: bool,
    dir: &'static str,
    users: Option<[User; 3]>
}

struct User {
    bin: &'static str,
    enrty: Option<usize>
}

const CH0: Makefile = Makefile {
    link_arg: "-Ttext=0x80200000",
    nightly: false,
    dir: "../ch0",
    users: None
};

const CH1: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch1",
    users: None
};

const CH2: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch2",
    users: Some([
        User {
            bin: "hello_world",
            enrty: Some(0x80400000)
        },
        User {
            bin: "priv_inst",
            enrty: Some(0x80400000)
        },
        User {
            bin: "store_fault",
            enrty: Some(0x80400000)
        }
    ])
};

const CH3: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch3",
    users: Some([
        User {
            bin: "00write_a",
            enrty: Some(0x80600000)
        },
        User {
            bin: "01write_b",
            enrty: Some(0x80620000)
        },
        User {
            bin: "02write_c",
            enrty: Some(0x80640000)
        }
    ])
};

const CH4: Makefile = Makefile {
    link_arg: "-Tsrc/linker.ld",
    nightly: true,
    dir: "../ch4",
    users: Some([
        User {
            bin: "00write_a",
            enrty: None
        },
        User {
            bin: "01write_b",
            enrty: None
        },
        User {
            bin: "02write_c",
            enrty: None
        }
    ])
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ch = args[1].as_str();

    let ch = match ch {
        "ch0" => CH0,
        "ch1" => CH1,
        "ch2" => CH2,
        "ch3" => CH3,
        "ch4" => CH4,
        _ => panic!()
    };

    
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    if let Some(users) = ch.users {
        clean("../user");
        for user in users {
            if let Some(entry) = user.enrty {
                let link_arg = format!("-Ttext={:x}", entry);
                let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg);
                build("../user", Some(&config), true, Some(user.bin));
            } else {
                build("../user", None, true, Some(user.bin));
            }
        }
    }
     
    clean(ch.dir);
    let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, ch.link_arg);
    build(ch.dir, Some(&config), ch.nightly, None);
    elf_to_bin(ch.dir, &kernel_elf, &kernel_bin);
    
    let output = qemu_run(ch.dir, &kernel_bin);
    print!("{}", output);
}
