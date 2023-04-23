#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;
use lib::getchar;
use lib::{exec, fork, waitpid};

#[no_mangle]
pub fn main() {
    println!("[shell]");
    let mut line: String = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    if line == "exit" {
                        return;
                    }
                    let pid = fork();
                    if pid == 0 {
                        // child process
                        if exec(line.as_str()) == 0 {
                            println!("[shell] Error when executing!");
                            return;
                        }
                        unreachable!();
                    } else {
                        waitpid(pid as usize);
                        println!("[shell]: Process {} exited", pid);
                    }
                    line.clear();
                }
                print!(">> ");
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}
