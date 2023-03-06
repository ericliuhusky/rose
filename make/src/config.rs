pub struct Makefile {
    pub link_arg: &'static str,
    pub nightly: bool,
    pub dir: &'static str,
    pub users: Vec<User>,
}

pub struct User {
    pub bin: &'static str,
    pub enrty: Option<usize>,
}

pub static mut CH: [Makefile; 6] = [
    Makefile {
        link_arg: "-Ttext=0x80200000",
        nightly: false,
        dir: "../ch0",
        users: Vec::new(),
    },
    Makefile {
        link_arg: "-Tsrc/linker.ld",
        nightly: true,
        dir: "../ch1",
        users: Vec::new(),
    },
    Makefile {
        link_arg: "-Tsrc/linker.ld",
        nightly: true,
        dir: "../ch2",
        users: Vec::new(),
    },
    Makefile {
        link_arg: "-Tsrc/linker.ld",
        nightly: true,
        dir: "../ch3",
        users: Vec::new(),
    },
    Makefile {
        link_arg: "-Tsrc/linker.ld",
        nightly: true,
        dir: "../ch4",
        users: Vec::new(),
    },
    Makefile {
        link_arg: "-Tsrc/linker.ld",
        nightly: true,
        dir: "../ch5",
        users: Vec::new(),
    },
];

pub fn init() {
    unsafe {
        CH[2].users = vec![
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
        ];

        CH[3].users = vec![
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
        ];

        CH[4].users = vec![
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
        ];

        CH[5].users = vec![
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
            User {
                bin: "sleep",
                enrty: None,
            },
        ];
    }
}

pub fn get_ch() -> &'static [Makefile; 6] {
    unsafe {
        &CH
    }
}