pub struct Makefile {
    pub link_arg: LinkArg,
    pub nightly: bool,
    pub dir: &'static str,
    pub users: Vec<User>,
}

pub struct User {
    pub bin: &'static str,
    pub link_arg: Option<LinkArg>,
}

pub enum LinkArg {
    Address(usize),
    File(&'static str),
}

static mut CH: Vec<Makefile> = Vec::new();

pub const TARGET: &str = "riscv64gc-unknown-none-elf";
pub const BOOTLOADER: &str = "../rustsbi-qemu.bin";
pub const KERNEL_ENTRY: &str = "0x80200000";

pub fn init() {
    unsafe {
        CH = vec![
            Makefile {
                link_arg: LinkArg::Address(0x80200000),
                nightly: false,
                dir: "../ch0",
                users: Vec::new(),
            },
            Makefile {
                link_arg: LinkArg::File("src/linker.ld"),
                nightly: true,
                dir: "../ch1",
                users: Vec::new(),
            },
            Makefile {
                link_arg: LinkArg::File("src/linker.ld"),
                nightly: true,
                dir: "../ch2",
                users: Vec::new(),
            },
            Makefile {
                link_arg: LinkArg::File("src/linker.ld"),
                nightly: true,
                dir: "../ch3",
                users: Vec::new(),
            },
            Makefile {
                link_arg: LinkArg::File("src/linker.ld"),
                nightly: true,
                dir: "../ch4",
                users: Vec::new(),
            },
            Makefile {
                link_arg: LinkArg::File("src/linker.ld"),
                nightly: true,
                dir: "../ch5",
                users: Vec::new(),
            },
        ];
        CH[2].users = vec![
            User {
                bin: "hello_world",
                link_arg: Some(LinkArg::Address(0x80400000)),
            },
            User {
                bin: "priv_inst",
                link_arg: Some(LinkArg::Address(0x80400000)),
            },
            User {
                bin: "store_fault",
                link_arg: Some(LinkArg::Address(0x80400000)),
            },
        ];

        CH[3].users = vec![
            User {
                bin: "00write_a",
                link_arg: Some(LinkArg::Address(0x80600000)),
            },
            User {
                bin: "01write_b",
                link_arg: Some(LinkArg::Address(0x80620000)),
            },
            User {
                bin: "02write_c",
                link_arg: Some(LinkArg::Address(0x80640000)),
            },
        ];

        CH[4].users = vec![
            User {
                bin: "00write_a",
                link_arg: None,
            },
            User {
                bin: "01write_b",
                link_arg: None,
            },
            User {
                bin: "02write_c",
                link_arg: None,
            },
        ];

        CH[5].users = vec![
            User {
                bin: "initproc",
                link_arg: None,
            },
            User {
                bin: "shell",
                link_arg: None,
            },
            User {
                bin: "fork",
                link_arg: None,
            },
            User {
                bin: "sleep",
                link_arg: None,
            },
        ];
    }
}

pub fn ch() -> &'static Vec<Makefile> {
    unsafe { &CH }
}
