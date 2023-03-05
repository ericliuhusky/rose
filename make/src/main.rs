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

fn build(dir: &str, config: &str, nightly: bool) {
    let mut cmd = Command::new("cargo");
    if nightly {
        cmd.arg("+nightly");
    }
    cmd
        .current_dir(dir) 
        .arg("build")
        .args(["--config", config])
        .args(["--target", TARGET])
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
        _ => ""
    };
    let nightly = match ch {
        "ch0" => false,
        "ch1" => true,
        _ => false
    };
    let dir = match ch {
        "ch0" => "../ch0",
        "ch1" => "../ch1",
        _ => ""
    };
    
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);
     
    clean(dir);
    let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg);
    build(dir, &config, nightly);
    elf_to_bin(dir, &kernel_elf, &kernel_bin);
    
    let output = qemu_run(dir, &kernel_bin);
    print!("{}", output);
}
