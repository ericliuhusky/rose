use super::busy_wait_udp_read;
use super::LOSE_NET_STACK;
use super::NET_DEVICE;
use crate::fs::File;
use crate::net::net_arp;
use alloc::vec;
use alloc::vec::Vec;
use lose_net_stack::packets::udp::UDPPacket;
use lose_net_stack::IPv4;
use lose_net_stack::MacAddress;

pub struct UDP {
    pub target: IPv4,
    pub sport: u16,
    pub dport: u16,
}

impl UDP {
    pub fn new(target: IPv4, sport: u16, dport: u16) -> Self {
        Self {
            target,
            sport,
            dport,
        }
    }
}

impl File for UDP {
    fn read(&mut self, mut buf: Vec<&'static mut [u8]>) -> usize {
        net_arp();
        let (data, source_ip, source_port) = busy_wait_udp_read(self.sport);
        self.target = source_ip;
        self.dport = source_port;

        println!("{}, {}", source_ip, source_port);

        let data_len = data.len();
        let mut left = 0;
        for i in 0..buf.len() {
            let buffer_i_len = buf[i].len().min(data_len - left);

            buf[i][..buffer_i_len]
                .copy_from_slice(&data[left..(left + buffer_i_len)]);

            left += buffer_i_len;
            if left == data_len {
                break;
            }
        }
        left
    }

    fn write(&mut self, buf: Vec<&'static mut [u8]>) -> usize {
        let lose_net_stack = LOSE_NET_STACK.0.borrow_mut();

        let mut data = vec![0u8; buf.concat().len()];

        let mut left = 0;
        for i in 0..buf.len() {
            data[left..(left + buf[i].len())].copy_from_slice(buf[i]);
            left += buf[i].len();
        }

        let len = data.len();

        let udp_packet = UDPPacket::new(
            lose_net_stack.ip,
            lose_net_stack.mac,
            self.sport,
            self.target,
            MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            self.dport,
            len,
            data.as_ref(),
        );
        NET_DEVICE.transmit(&udp_packet.build_data());
        len
    }
}
