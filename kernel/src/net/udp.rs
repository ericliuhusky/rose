use super::busy_wait_udp_read;
use super::LOCALHOST_IP;
use super::LOCALHOST_MAC;
use super::NET_DEVICE;
use crate::fs::File;
use crate::net::net_arp;
use alloc::vec;
use lose_net_stack::packets::udp::UDPPacket;
use lose_net_stack::IPv4;
use lose_net_stack::MacAddress;
use page_table::PhysicalBufferList;

pub struct UDP {
    pub source_ip: IPv4,
    pub source_mac: MacAddress,
    pub source_port: u16,
    pub dest_ip: IPv4,
    pub dest_mac: MacAddress,
    pub dest_port: u16,
}

impl UDP {
    pub fn new() -> Self {
        Self {
            source_ip: LOCALHOST_IP,
            source_mac: LOCALHOST_MAC,
            source_port: 0,
            dest_ip: IPv4::from_u32(0),
            dest_mac: MacAddress::new([0; 6]),
            dest_port: 0,
        }
    }
}

impl File for UDP {
    fn read(&mut self, mut buf: PhysicalBufferList) -> usize {
        net_arp();
        let (data, source_ip, source_mac, source_port) = busy_wait_udp_read(self.source_port);
        self.dest_ip = source_ip;
        self.dest_mac = source_mac;
        self.dest_port = source_port;


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

        let udp_packet = UDPPacket::new(
            self.source_ip,
            self.source_mac,
            self.source_port,
            self.dest_ip,
            self.dest_mac,
            self.dest_port,
            len,
            data.as_ref(),
        );
        NET_DEVICE.transmit(&udp_packet.build_data());
        len
    }

    fn file_type(&self) -> crate::fs::FileType {
        crate::fs::FileType::UDP
    }
}
