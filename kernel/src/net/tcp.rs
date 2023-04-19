use super::TransPort;
use super::busy_wait_tcp_read;
use crate::fs::File;
use alloc::vec;
use super::Eth;
use super::Ip;
use super::TCPHeader;
use super::TcpFlags;
use page_table::PhysicalBufferList;

// add tcp packet info to this structure
pub struct TCP {
    pub source_port: u16,
    pub eth: Eth,
    pub ip: Ip,
    pub tcp: TCPHeader,
}

impl TCP {
    pub fn new_server() -> Self {
        Self {
            source_port: 0,
            eth: Eth::default(),
            ip: Ip::default(),
            tcp: TCPHeader::default(),
        }
    }

    pub fn new(
        dest_port: u16,
        eth: Eth,
        ip: Ip,
        tcp: TCPHeader,
    ) -> Self {
        Self {
            source_port: dest_port,
            eth,
            ip,
            tcp,
        }
    }
}

impl File for TCP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        let (eth, ip, tcp, data) = busy_wait_tcp_read(self.source_port);
        self.tcp = tcp;

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

        let (seq, ack) = (self.tcp.seq, self.tcp.ack);
        self.tcp.ack = seq;
        self.tcp.seq = ack;
        self.tcp.flags = TcpFlags::A;

        TransPort::send_tcp(self.eth, self.ip, self.tcp, data.clone());

        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::TCP
    }
}
