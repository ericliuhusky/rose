const TARGET: &str = "riscv64gc-unknown-none-elf";
const BOOTLOADER: &str = "../rustsbi-qemu.bin";
const KERNEL_ENTRY: &str = "0x80200000";

use std::process::Command;

fn clean(ch: &str) {
    Command::new("cargo")
        .current_dir(format!("../{}", ch))
        .arg("clean")
        .spawn()
        .expect("clean")
        .wait()
        .expect("wait clean");
}

fn build(ch: &str, config: &str, nightly: bool) {
    let mut cmd = Command::new("cargo");
    if nightly {
        cmd.arg("+nightly");
    }
    cmd
        .current_dir(format!("../{}", ch)) 
        .arg("build")
        .args(["--config", config])
        .args(["--target", TARGET])
        .arg("--release")
        .spawn()
        .expect("build")
        .wait()
        .expect("wait build");
}

fn elf_to_bin(ch: &str, kernel_elf: &str, kernel_bin: &str) {
    Command::new("rust-objcopy")
        .current_dir(format!("../{}", ch))
        .arg(kernel_elf)
        .arg("--strip-all")
        .args(["-O", "binary", kernel_bin])
        .spawn()
        .expect("elf_to_bin")
        .wait()
        .expect("wait elf_to_bin");
}

fn qemu_run(ch: &str, kernel_bin: &str) -> String {
    Command::new("qemu-system-riscv64")
        .current_dir(format!("../{}", ch))
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
    
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);
     
    clean(&ch);
    let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, link_arg);
    build(&ch, &config, nightly);
    elf_to_bin(&ch, &kernel_elf, &kernel_bin);
    
    let output = qemu_run(&ch, &kernel_bin);
    print!("{}", output);
}
