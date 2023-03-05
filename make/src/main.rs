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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ch = args[1].as_str();
    let link_arg = match ch {
        "ch0" => "-Ttext=0x80200000",
        "ch1" => "-Tsrc/linker.ld",
        "ch2" => "-Tsrc/linker.ld",
        "ch3" => "-Tsrc/linker.ld",
        "ch4" => "-Tsrc/linker.ld",
        _ => ""
    };
    let nightly = match ch {
        "ch0" => false,
        "ch1" => true,
        "ch2" => true,
        "ch3" => true,
        "ch4" => true,        
        _ => false
    };
    let dir = match ch {
        "ch0" => "../ch0",
        "ch1" => "../ch1",
        "ch2" => "../ch2",
        "ch3" => "../ch3",
        "ch4" => "../ch4",
        _ => ""
    };
    let has_user = match ch {
        "ch0" => false,
        "ch1" => false,
        "ch2" => true,
        "ch3" => true,
        "ch4" => true,
        _ => false
    };
    let link_arg_users = match ch {
        "ch0" => None,
        "ch1" => None,
        "ch2" => Some(vec!["-Ttext=0x80400000"]),
        "ch3" => Some(vec!["-Ttext=0x80600000", "-Ttext=0x80620000", "-Ttext=0x80640000"]),
        "ch4" => None,
        _ => None
    };
    
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);

    if has_user {
        clean("../user");
        if let Some(link_arg_users) = link_arg_users {
            if link_arg_users.len() == 1 {
                let config_user = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg_users[0]);
                build("../user", Some(&config_user), true, None);
            } else {
                let config_user = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg_users[0]);
                build("../user", Some(&config_user), true, Some("00write_a"));
                let config_user = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg_users[1]);
                build("../user", Some(&config_user), true, Some("01write_b"));
                let config_user = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg_users[2]);
                build("../user", Some(&config_user), true, Some("02write_c"));
            }
            
        } else {
            build("../user", None, true, None);
        }
    }
     
    clean(dir);
    let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg);
    build(dir, Some(&config), nightly, None);
    elf_to_bin(dir, &kernel_elf, &kernel_bin);
    
    let output = qemu_run(dir, &kernel_bin);
    print!("{}", output);
}
