#![no_std]

mod net;
mod addr;
mod consts;
pub mod packets;
pub mod results;
pub(crate) mod utils;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate print;

pub use addr::IPv4;
pub use addr::MacAddress;
pub use net::TcpFlags;
pub use net::Eth;
pub use net::EthType;
pub use net::Ip;
pub use net::IPProtocal;
use net::TCP;
use net::IP_LEN;
use results::Packet;
pub use utils::UnsafeRefIter;
use consts::*;
pub use utils::check_sum;

use crate::net::TCP_LEN;

pub struct LoseStack {
    pub ip:  IPv4,
    pub mac: MacAddress
}

impl LoseStack {
    pub const fn new(ip: IPv4, mac: MacAddress) -> Self {
        Self {
            ip,
            mac
        }
    }

    fn analysis_tcp(&self, mut data_ptr_iter: UnsafeRefIter, ip_header: &Ip, eth_header: &Eth) -> Packet {
        let tcp_header = unsafe{data_ptr_iter.next::<TCP>()}.unwrap();
        let offset = ((tcp_header.offset >> 4 & 0xf) as usize - 5) * 4;
        let data = &unsafe{data_ptr_iter.get_curr_arr()}[offset..];
        let data_len = ip_header.len.to_be() as usize - TCP_LEN - IP_LEN - offset;

        Packet::TCP(packets::tcp::TCPPacket {
            source_ip: IPv4::from_u32(ip_header.src.to_be()), 
            source_mac: MacAddress::new(eth_header.shost), 
            source_port: tcp_header.sport.to_be(), 
            dest_ip: IPv4::from_u32(ip_header.dst.to_be()), 
            dest_mac: MacAddress::new(eth_header.dhost), 
            dest_port: tcp_header.dport.to_be(), 
            data_len,
            seq: tcp_header.seq.to_be(),
            ack: tcp_header.ack.to_be(),
            flags: tcp_header.flags,
            win: tcp_header.win.to_be(),
            urg: tcp_header.urg.to_be(),
            data: data.to_vec(),
        })
    }

    fn analysis_ip(&self, mut data_ptr_iter: UnsafeRefIter, eth_header: &Eth) -> Packet {
        let ip_header = unsafe{data_ptr_iter.next::<Ip>()}.unwrap();

        if ip_header.dst.to_be() != self.ip.to_u32() {
            return Packet::None
        }

        match ip_header.pro {
            IP_PROTOCAL_TCP => self.analysis_tcp(data_ptr_iter, ip_header, eth_header),
            _ => Packet::None,
        }
    }

    pub fn analysis(&self, data: &[u8]) -> Packet {
        let mut data_ptr_iter = UnsafeRefIter::new(data);
        let eth_header = unsafe{data_ptr_iter.next::<Eth>()}.unwrap();
        match eth_header.type_() {
            EthType::IP => self.analysis_ip(data_ptr_iter, eth_header),
            _ => panic!(),
        }
    }
}
