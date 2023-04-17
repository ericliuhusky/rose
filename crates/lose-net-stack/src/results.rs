use alloc::vec::Vec;

use crate::{Eth, Ip};

use super::net::TCPHeader;

pub enum Packet {
    TCP((Eth, Ip, TCPHeader, Vec<u8>)),
    None
}
