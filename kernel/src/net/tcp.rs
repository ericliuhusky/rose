use super::busy_wait_tcp_read;
use super::LOCALHOST_IP;
use super::LOCALHOST_MAC;
use crate::{drivers::virtio_net::NET_DEVICE, fs::File};
use alloc::vec;
use lose_net_stack::packets::tcp::TCPPacket;
use lose_net_stack::IPv4;
use lose_net_stack::MacAddress;
use lose_net_stack::TcpFlags;
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
}

impl TCP {
    pub fn new_server() -> Self {
        Self {
            source_ip: *LOCALHOST_IP,
            source_mac: *LOCALHOST_MAC,
            source_port: 0,
            dest_ip: IPv4::from_u32(0),
            dest_mac: MacAddress::new([0; 6]),
            dest_port: 0,
            seq: 0,
            ack: 0,
        }
    }

    pub fn new(
        source_ip: IPv4,
        source_mac: MacAddress,
        source_port: u16,
        dest_port: u16,
        seq: u32,
        ack: u32,
    ) -> Self {
        Self {
            source_ip: *LOCALHOST_IP,
            source_mac: *LOCALHOST_MAC,
            source_port: dest_port,
            dest_ip: source_ip,
            dest_mac: source_mac,
            dest_port: source_port,
            seq,
            ack,
        }
    }
}

impl File for TCP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        let (data, seq, ack) = busy_wait_tcp_read(self.source_port, self.dest_ip, self.dest_port);
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

        let (ack, seq) = (self.seq, self.ack);

        let tcp_packet = TCPPacket {
            source_ip: self.source_ip,
            source_mac: self.source_mac,
            source_port: self.source_port,
            dest_ip: self.dest_ip,
            dest_mac: self.dest_mac,
            dest_port: self.dest_port,
            data_len: len,
            seq,
            ack,
            flags: TcpFlags::A,
            win: 65535,
            urg: 0,
            data: data.as_ref(),
        };
        NET_DEVICE.transmit(&tcp_packet.build_data());
        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::TCP
    }
}
