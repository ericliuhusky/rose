use super::config::{Makefile, TARGET};
use std::{fs::File, io::Write};

pub fn create_link_app(ch: &Makefile) {
    if ch.users.is_empty() {
        return;
    }
    let mut f = File::create(format!("{}/src/link_app.s", ch.dir)).unwrap();
    writeln!(
        f,
        r#"# created by crate make
    .section .data
    .globl _num_app
_num_app:
    .quad {}"#,
        ch.users.len()
    )
    .unwrap();
    for i in 0..ch.users.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i).unwrap();
    }
    writeln!(f, r#"    .quad app_{}_end"#, ch.users.len() - 1).unwrap();
    writeln!(
        f,
        r#"
    .globl _app_names
_app_names:"#
    )
    .unwrap();
    for user in &ch.users {
        writeln!(f, r#"    .string "{}""#, user.bin).unwrap();
    }
    for (i, user) in ch.users.iter().enumerate() {
        writeln!(
            f,
            r#"
    .section .data
    .globl app_{0}_start
    .globl app_{0}_end
app_{0}_start:
    .incbin "../user/target/{1}/release/{2}"
app_{0}_end:"#,
            i, TARGET, user.bin
        )
        .unwrap();
    }
}
