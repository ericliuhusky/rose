const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn clean(dir: &str) {
    Command::new("cargo")
        .current_dir(dir)
        .arg("clean")
        .arg("--quiet")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn build(dir: &str, nightly: bool, config: Option<&str>, bin: Option<&str>) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir);
    if nightly {
        cmd.arg("+nightly");
    }
    cmd.arg("build");
    if let Some(config) = config {
        cmd.args(["--config", config]);
    }
    cmd.args(["--target", TARGET]);
    if let Some(bin) = bin {
        cmd.args(["--bin", bin]);
    }
    cmd.arg("--release")
        .arg("--quiet")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn elf_to_bin(dir: &str, kernel_elf: &str, kernel_bin: &str) {
    Command::new("rust-objcopy")
        .current_dir(dir)
        .arg(kernel_elf)
        .arg("--strip-all")
        .args(["-O", "binary", kernel_bin])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn qemu_run(dir: &str, kernel_bin: &str) {
    let mut child = Command::new("qemu-system-riscv64")
        .current_dir(dir)
        .args(["-machine", "virt"])
        .arg("-nographic")
        .args(["-bios", BOOTLOADER])
        .args([
            "-device",
            &format!("loader,file={},addr={}", kernel_bin, KERNEL_ENTRY),
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stdin = child.stdin.take().unwrap();

    let out_thread = thread::spawn(move || loop {
        let mut buf = [0; 0x1000];
        let len = child_stdout.read(&mut buf).unwrap();
        if len == 0 {
            break;
        }
        io::stdout().write_all(&buf).unwrap();
        thread::sleep(Duration::from_millis(10));
    });
    let in_thread = thread::spawn(move || loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        child_stdin.write_all(buf.as_bytes()).unwrap();
        thread::sleep(Duration::from_millis(10));
    });

    out_thread.join().unwrap();
    in_thread.join().unwrap();
    child.wait().unwrap();
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
    let args: Vec<String> = std::env::args().collect();
    let ch = args[1].as_str();

    let ch = match ch {
        "ch0" => CH0,
        "ch1" => CH1,
        "ch2" => CH2,
        "ch3" => CH3,
        "ch4" => CH4,
        "ch5" => CH5,
        _ => panic!(),
    };

    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    fn rustflags(link_arg: &str) -> String {
        format!(
            r#"target.{}.rustflags = ["-Clink-arg={}"]"#,
            TARGET, link_arg
        )
    }

    if let Some(users) = ch.users {
        clean("../user");
        for user in users {
            if let Some(entry) = user.enrty {
                let link_arg = format!("-Ttext={:x}", entry);
                let config = rustflags(&link_arg);
                build("../user", true, Some(&config), Some(user.bin));
            } else {
                build("../user", true, None, Some(user.bin));
            }
        }
    }

    clean(ch.dir);
    let config = rustflags(ch.link_arg);
    build(ch.dir, ch.nightly, Some(&config), None);
    elf_to_bin(ch.dir, &kernel_elf, &kernel_bin);

    qemu_run(ch.dir, &kernel_bin);
}
