fn main() {
    use std::{env, fs, path::PathBuf};
    let ld = &PathBuf::from(env::var("OUT_DIR").unwrap()).join("linker.ld");
    fs::write(ld, LINKER).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LOG");
    println!("cargo:rustc-link-arg=-T{}", ld.display());
}

const LINKER: &[u8] = b"
SECTIONS {
    . = 0x80200000;

    skernel = .;
    .text : {
        *(.text.entry)
        strampoline = .;
        *(.text.trampoline)
        etrampoline = .;
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ekernel = .;
}";
