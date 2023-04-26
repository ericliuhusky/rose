use super::TCPPacket;
use super::TransPort;
use super::busy_wait_tcp_read;
use crate::fs::File;
use alloc::vec;
use super::TcpFlags;
use page_table::PhysicalBufferList;

// add tcp packet info to this structure
pub struct TCP {
    pub source_port: u16,
    tcp: Option<TCPPacket>,
}

impl TCP {
    pub fn new_server() -> Self {
        Self {
            source_port: 0,
            tcp: None,
        }
    }

    pub fn new(
        dest_port: u16,
    ) -> Self {
        Self {
            source_port: dest_port,
            tcp: None,
        }
    }
}

impl File for TCP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        let (tcp, data_len) = busy_wait_tcp_read(self.source_port);
        self.tcp = Some(tcp.clone());
        let offset = ((tcp.tcp.offset >> 4 & 0xf) as usize - 5) * 4;
        let data = &tcp.data[offset..data_len];

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

        let tcp = self.tcp.as_ref().unwrap().clone();

        let mut tcp_h = tcp.tcp;

        let (seq, ack) = (tcp_h.seq, tcp_h.ack);
        tcp_h.ack = seq;
        tcp_h.seq = ack;
        tcp_h.flags = TcpFlags::A;

        TransPort::send_tcp(tcp.eth, tcp.ip, tcp_h, data.clone());

        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::TCP
    }
}
