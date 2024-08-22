use super::UDPPacket;
use super::net::{recv_udp, send_udp};
use crate::fs::FileInterface;
use crate::net::net_arp;
use alloc::vec;
use page_table::PhysicalBufferList;

pub struct UDP {
    pub source_port: u16,
    udp: UDPPacket,
}

impl UDP {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            udp: UDPPacket::default(),
        }
    }
}

impl FileInterface for UDP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        net_arp();
        let udp: UDPPacket;
        loop {
            if let Some(_udp) = recv_udp(self.source_port) {
                udp = _udp;
                break;
            }
        }

        let data = udp.data.clone();

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
        self.udp.data = data;
        send_udp(self.udp.clone());
        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::UDP
    }
}
