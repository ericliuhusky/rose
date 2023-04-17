use alloc::vec::Vec;

use crate::{Eth, Ip};

use super::net::TCP;

pub enum Packet {
    TCP((Eth, Ip, TCP, Vec<u8>)),
    None
}
