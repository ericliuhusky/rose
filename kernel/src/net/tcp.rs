use alloc::vec;
use alloc::vec::Vec;
use lose_net_stack::packets::tcp::TCPPacket;
use lose_net_stack::IPv4;
use lose_net_stack::MacAddress;
use lose_net_stack::TcpFlags;

use crate::{drivers::virtio_net::NET_DEVICE, fs::File};

use super::busy_wait_tcp_read;
use super::{
    LOSE_NET_STACK,
};

// add tcp packet info to this structure
pub struct TCP {
    pub target: IPv4,
    pub sport: u16,
    pub dport: u16,
    pub seq: u32,
    pub ack: u32,
}

impl TCP {
    pub fn new(target: IPv4, sport: u16, dport: u16, seq: u32, ack: u32) -> Self {
        Self {
            target,
            sport,
            dport,
            seq,
            ack,
        }
    }
}

impl File for TCP {
    fn read(&mut self, mut buf: Vec<&'static mut [u8]>) -> usize {
        let (data, seq, ack) = busy_wait_tcp_read(self.sport, self.target, self.dport);
        self.seq = seq;
        self.ack = ack;
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

        let (ack, seq) = (self.seq, self.ack);

        let tcp_packet = TCPPacket {
            source_ip: lose_net_stack.ip,
            source_mac: lose_net_stack.mac,
            source_port: self.sport,
            dest_ip: self.target,
            dest_mac: MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            dest_port: self.dport,
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
}

