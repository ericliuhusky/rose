use super::TransPort;
use super::busy_wait_tcp_read;
use super::LOCALHOST_IP;
use super::LOCALHOST_MAC;
use crate::{drivers::virtio_net::NET_DEVICE, fs::File};
use alloc::vec;
use super::Eth;
use super::Ip;
use super::TCPHeader;
use super::IPv4;
use super::MacAddress;
use super::TcpFlags;
use page_table::PhysicalBufferList;

// add tcp packet info to this structure
pub struct TCP {
    pub source_ip: IPv4,
    pub source_mac: MacAddress,
    pub source_port: u16,
    pub dest_ip: IPv4,
    pub dest_mac: MacAddress,
    pub dest_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub eth: Eth,
    pub ip: Ip,
    pub tcp: TCPHeader,
}

impl TCP {
    pub fn new_server() -> Self {
        Self {
            source_ip: LOCALHOST_IP,
            source_mac: LOCALHOST_MAC,
            source_port: 0,
            dest_ip: IPv4::default(),
            dest_mac: MacAddress::default(),
            dest_port: 0,
            seq: 0,
            ack: 0,
            eth: Eth::default(),
            ip: Ip::default(),
            tcp: TCPHeader::default(),
        }
    }

    pub fn new(
        source_ip: IPv4,
        source_mac: MacAddress,
        source_port: u16,
        dest_port: u16,
        seq: u32,
        ack: u32,
        eth: Eth,
        ip: Ip,
        tcp: TCPHeader,
    ) -> Self {
        Self {
            source_ip: LOCALHOST_IP,
            source_mac: LOCALHOST_MAC,
            source_port: dest_port,
            dest_ip: source_ip,
            dest_mac: source_mac,
            dest_port: source_port,
            seq,
            ack,
            eth,
            ip,
            tcp,
        }
    }
}

impl File for TCP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        let (data, seq, ack) = busy_wait_tcp_read(self.source_port);
        self.seq = seq;
        self.ack = ack;

        for (i, byte) in buf.iter_mut().enumerate() {
            if i >= data.len() {
                return i;
            }
            *byte = data[i];
        }
        buf.len()
    }

    fn write(&mut self, buf: PhysicalBufferList) -> usize {
        let mut data = vec![0u8; buf.len()];

        for (i, byte) in buf.iter().enumerate() {
            data[i] = byte;
        }

        let len = data.len();

        self.tcp.ack = self.seq.to_be();
        self.tcp.seq = self.ack.to_be();
        self.tcp.flags = TcpFlags::A;

        TransPort::send_tcp(self.eth, self.ip, self.tcp, data.clone());

        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::TCP
    }
}
