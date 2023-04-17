use crate::{packets::{tcp::TCPPacket}};

pub enum Packet {
    TCP(TCPPacket<'static>),
    None
}
