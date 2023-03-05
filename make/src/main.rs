const TARGET: &str = "riscv64gc-unknown-none-elf";
const LINK_ARG: &str = "-Ttext=0x80200000";
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

fn build(ch: &str, config: &str) {
    Command::new("cargo")
        .current_dir(format!("../{}", ch))
        .arg("build")
        .arg("--config")
        .arg(config)
        .arg("--target")
        .arg(TARGET)
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
        .arg("-O")
        .arg("binary")
        .arg(kernel_bin)
        .spawn()
        .expect("elf_to_bin")
        .wait()
        .expect("wait elf_to_bin");
}

fn qemu_run(ch: &str, kernel_bin: &str) -> String {
    Command::new("qemu-system-riscv64")
        .current_dir(format!("../{}", ch))
        .arg("-machine")
        .arg("virt")
        .arg("-nographic")
        .arg("-bios")
        .arg(BOOTLOADER)
        .arg("-device")
        .arg(format!("loader,file={},addr={}", kernel_bin, KERNEL_ENTRY))
        .output()
        .expect("qemu_run")
        .stdout
        .iter().map(|b| *b as char).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ch = &args[1];
    
    let kernel_elf = format!("target/{}/release/kernel", TARGET);
    let kernel_bin = format!("{}.bin", kernel_elf);
     
    clean(&ch);
    let config = format!(r#"target.{}.rustflags = ["-Clink-arg={}"]"#, TARGET, LINK_ARG);
    build(&ch, &config);
    elf_to_bin(&ch, &kernel_elf, &kernel_bin);
    
    let output = qemu_run(&ch, &kernel_bin);
    print!("{}", output);
}
