use super::TransPort;
use crate::fs::File;
use crate::net::net_arp;
use alloc::vec;
use alloc::vec::Vec;
use super::Eth;
use super::Ip;
use super::UDPHeader;
use page_table::PhysicalBufferList;

pub struct UDP {
    pub source_port: u16,
    pub eth: Eth,
    pub ip: Ip,
    pub udp: UDPHeader,
}

impl UDP {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            eth: Eth::default(),
            ip: Ip::default(),
            udp: UDPHeader::default(),
        }
    }
}

impl File for UDP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        net_arp();
        let (eth, ip, udp, data): (Eth, Ip, UDPHeader, Vec<u8>);
        loop {
            if let Some((_eth, _ip, _udp, _data)) = TransPort::recv_udp(self.source_port) {
                eth = _eth;
                ip = _ip;
                udp = _udp;
                data = _data;
                break;
            }
        }

        self.eth = eth;
        self.ip = ip;
        self.udp = udp;

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
        TransPort::send_udp(self.eth, self.ip,self.udp, data);
        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::UDP
    }
}
