use crate::{packets::{udp::UDPPacket, tcp::TCPPacket}};

pub enum Packet {
    UDP(UDPPacket),
    TCP(TCPPacket<'static>),
    None
}

#[derive(Debug)]
pub enum NetStackErrors {
    NotRequiredReplyArp,
}