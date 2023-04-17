use super::TransPort;
use crate::fs::File;
use crate::net::net_arp;
use alloc::vec;
use lose_net_stack::Eth;
use lose_net_stack::Ip;
use page_table::PhysicalBufferList;

pub struct UDP {
    pub source_port: u16,
    pub eth: Eth,
    pub ip: Ip,
    pub udp: lose_net_stack::UDP,
}

impl UDP {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            eth: Eth::default(),
            ip: Ip::default(),
            udp: lose_net_stack::UDP::default(),
        }
    }
}

impl File for UDP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        net_arp();
        let (eth, ip, udp, data) = TransPort::recv_udp(self.source_port);
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
