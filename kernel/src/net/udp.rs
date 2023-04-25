use super::TransPort;
use super::UDPPacket;
use crate::fs::File;
use crate::net::net_arp;
use alloc::vec;
use page_table::PhysicalBufferList;

pub struct UDP {
    pub source_port: u16,
    udp: Option<UDPPacket>,
}

impl UDP {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            udp: None,
        }
    }
}

impl File for UDP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        net_arp();
        let (udp, data_len): (UDPPacket, usize);
        loop {
            if let Some((_udp, _data_len)) = TransPort::recv_udp(self.source_port) {
                udp = _udp;
                data_len = _data_len;
                break;
            }
        }

        let data = udp.data[..data_len].to_vec();

        self.udp = Some(udp);

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
        TransPort::send_udp(self.udp.take().unwrap(), data);
        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::UDP
    }
}
