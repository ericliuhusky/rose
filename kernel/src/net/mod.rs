pub mod port_table;
pub mod tcp;
pub mod udp;

use core::mem::size_of;
use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::vec;
use alloc::vec::Vec;
use lose_net_stack::{Eth, EthType, Ip, IPProtocal, check_sum, TCPHeader};
pub use lose_net_stack::IPv4;
use lose_net_stack::{MacAddress, TcpFlags};

use self::tcp::TCP;

pub const LOCALHOST_IP: IPv4 = IPv4::new(10, 0, 2, 15);
pub const LOCALHOST_MAC: MacAddress = MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);

// pub fn net_interrupt_handler() {
//     let mut recv_buf = vec![0u8; 1024];

//     let len = NET_DEVICE.receive(&mut recv_buf);

//     let packet = LOSE_NET_STACK.analysis(&recv_buf[..len]);

//     match packet {
//         Packet::TCP(tcp_packet) => {
//             let target = tcp_packet.source_ip;
//             let lport = tcp_packet.dest_port;
//             let rport = tcp_packet.source_port;
//             let flags = tcp_packet.flags;

//             if flags.contains(TcpFlags::F) {
//                 // tcp disconnected
//                 let reply_packet = tcp_packet.ack();
//                 NET_DEVICE.transmit(&reply_packet.build_data());

//                 let mut end_packet = reply_packet.ack();
//                 end_packet.flags |= TcpFlags::F;
//                 NET_DEVICE.transmit(&end_packet.build_data());
//             }
//         }
//         _ => {}
//     }
// }

pub fn net_arp() {
    if let Some((eth, arp)) = Net::recv_arp() {
        Net::send_arp(eth, arp);
    }
}

pub fn net_accept(lport: u16) -> Option<TCP> {
    if let Some((eth, ip, tcp, data)) = TransPort::recv_tcp(lport) {    
        if tcp.flags.contains(TcpFlags::S) {
            TransPort::send_tcp(eth, ip, tcp.ack(), vec![]);
    
            Some(TCP::new(
                IPv4::from_u32(ip.src.to_be()),
                MacAddress::new(eth.shost),
                tcp.sport.to_be(),
                tcp.dport.to_be(),
                tcp.seq.to_be(),
                tcp.ack.to_be(),
                eth,
                ip,
                tcp,
            ))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn net_tcp_read(lport: u16) -> Option<(Vec<u8>, u32, u32)> {
    if let Some((eth, ip, tcp, data)) = TransPort::recv_tcp(lport) {
        if tcp.flags.contains(TcpFlags::A) {
            if data.len() == 0 {
                return None;
            }
            Some((data, tcp.seq.to_be(), tcp.ack.to_be()))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn busy_wait_tcp_read(lport: u16) -> (Vec<u8>, u32, u32) {
    loop {
        if let Some(data) = net_tcp_read(lport) {
            return data;
        }
    }
}

pub fn busy_wait_accept(lport: u16) -> TCP {
    loop {
        if let Some(socket) = net_accept(lport) {
            return socket;
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum ArpType {
    Request,
    Reply,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Arp {
    pub httype: u16, // Hardware type
    pub pttype: u16, // Protocol type, For IPv4, this has the value 0x0800.
    pub hlen: u8,    // Hardware length: Ethernet address length is 6.
    pub plen: u8,    // Protocol length: IPv4 address length is 4.
    op: u16,     // Operation: 1 for request, 2 for reply.
    pub sha: [u8; 6],// Sender hardware address
    pub spa: u32,    // Sender protocol address
    pub tha: [u8; 6],// Target hardware address
    pub tpa: u32     // Target protocol address
}

impl Arp {
    pub fn type_(&self) -> ArpType {
        match self.op.to_be() {
            1 => ArpType::Request,
            2 => ArpType::Reply,
            _ => panic!()
        }
    }

    pub fn set_type(&mut self, type_: ArpType) {
        let op: u16 = match type_ {
            ArpType::Request => 1,
            ArpType::Reply => 2,
        };
        self.op = op.to_be();
    }

    pub fn src_ip(&self) -> IPv4 {
        IPv4::from_u32(self.spa.to_be())
    }

    pub fn src_mac(&self) -> MacAddress {
        MacAddress::new(self.sha)
    }

    pub fn set_src_ip(&mut self, ip: IPv4) {
        self.spa = ip.to_u32().to_be()
    }

    pub fn set_src_mac(&mut self, mac: MacAddress) {
        self.sha = mac.to_bytes()
    }

    pub fn set_dst_ip(&mut self, ip: IPv4) {
        self.tpa = ip.to_u32().to_be()
    }

    pub fn set_dst_mac(&mut self, mac: MacAddress) {
        self.tha = mac.to_bytes()
    }
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UDPHeader {
    pub sport: u16, // souce port
    pub dport: u16, // destination port
    pub ulen: u16,  // length, including udp header, not including IP header
    pub sum: u16    // checksum
}

struct Link;

impl Link {
    fn recv_eth() -> (Eth, Vec<u8>) {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let eth_len = size_of::<Eth>();
        let remain_data = &data[eth_len..];
        let eth = unsafe { &*(&data[..eth_len] as *const _ as *const Eth) };

        (*eth, remain_data.to_vec())
    }

    fn send_eth(eth: Eth, data: Vec<u8>) {
        let mut re_eth = eth;
        re_eth.dhost = eth.shost;
        re_eth.shost = LOCALHOST_MAC.to_bytes();

        let eth_data = unsafe { &*(&re_eth as *const Eth as *const [u8; 14]) };
        let eth_data = eth_data.to_vec();
        let mut reply_data = eth_data;
        reply_data.extend(data);

        NET_DEVICE.transmit(&reply_data)
    }
}

struct Net;

impl Net {
    fn recv_arp() -> Option<(Eth, Arp)> {
        let (eth, data) = Link::recv_eth();
        if eth.type_() == EthType::ARP {
            let arp = unsafe { &*(&data[..] as *const [u8] as *const Arp) };
            Some((eth, *arp))
        } else {
            None
        }
    }

    fn send_arp(eth: Eth, arp: Arp) {        
        let mut re_arp = arp;
        re_arp.set_src_ip(LOCALHOST_IP);
        re_arp.set_src_mac(LOCALHOST_MAC);
        re_arp.set_dst_ip(arp.src_ip());
        re_arp.set_dst_mac(arp.src_mac());
        re_arp.set_type(ArpType::Reply);

        let data = unsafe { &*(&re_arp as *const Arp as *const [u8; 28]) };
        let data = data.to_vec();

        Link::send_eth(eth, data);
    }

    fn recv_ip() -> Option<(Eth, Ip, Vec<u8>)> {
        let (eth, data) = Link::recv_eth();
        if eth.type_() == EthType::IP {
            let ip_len = size_of::<Ip>();
            let ip = unsafe { &*(&data[..ip_len] as *const [u8] as *const Ip) };
            let remain_data = &data[ip_len..];
            Some((eth, *ip, remain_data.to_vec()))
        } else {
            None
        }
    }

    fn send_ip(eth: Eth, ip: Ip, data: Vec<u8>) {
        let len = data.len() + size_of::<Ip>();
        let mut re_ip = ip;
        re_ip.src = ip.dst;
        re_ip.dst = ip.src;
        re_ip.len = (len as u16).to_be();
        re_ip.sum = 0;
        re_ip.sum = check_sum(&mut re_ip as *mut Ip as *mut u8, size_of::<Ip>() as _, 0);

        let header_data = unsafe { &*(&re_ip as *const Ip as *const [u8; 20]) };
        let header_data = header_data.to_vec();

        let mut total_data = header_data;
        total_data.extend(data);
        Link::send_eth(eth, total_data)
    }
}

struct TransPort;

impl TransPort {
    fn recv_udp(port: u16) -> Option<(Eth, Ip, UDPHeader, Vec<u8>)> {
        if let Some((eth, ip, data)) = Net::recv_ip() {
            if ip.protocol() == IPProtocal::UDP {
                let udp_len = size_of::<UDPHeader>();
                let udp = unsafe { &*(&data[..udp_len] as *const [u8] as *const UDPHeader) };
                if udp.dport.to_be() == port {
                    let remain_data = &data[udp_len..];
                    Some((eth, ip, *udp, remain_data.to_vec()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn send_udp(eth: Eth, ip: Ip, udp: UDPHeader, data: Vec<u8>) {
        let len = data.len() + size_of::<UDPHeader>();
        let mut re_udp = udp;
        re_udp.sport = udp.dport;
        re_udp.dport = udp.sport;
        re_udp.sum = 0;
        re_udp.ulen = (len as u16).to_be();

        let header_data = unsafe { &*(&re_udp as *const UDPHeader as *const [u8; 8]) };
        let header_data = header_data.to_vec();
        let mut total_data = header_data;
        total_data.extend(data);

        Net::send_ip(eth, ip, total_data);
    }

    fn recv_tcp(port: u16) -> Option<(Eth, Ip, TCPHeader, Vec<u8>)> {
        if let Some((eth, ip, data)) = Net::recv_ip() {
            if ip.protocol() == IPProtocal::TCP {
                let tcp_len = size_of::<TCPHeader>();
                let tcp = unsafe { &*(&data[..tcp_len] as *const [u8] as *const TCPHeader) };
                if tcp.dport.to_be() == port {
                    let remain_data = &data[tcp_len..];
                    let offset = ((tcp.offset >> 4 & 0xf) as usize - 5) * 4;
                    let offset_data = &remain_data[offset..];
                    Some((eth, ip, *tcp, offset_data.to_vec()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn send_tcp(eth: Eth, ip: Ip, tcp: TCPHeader, data: Vec<u8>) {
        let mut re_tcp = tcp;
        re_tcp.sport = tcp.dport;
        re_tcp.dport = tcp.sport;
        re_tcp.offset = 5 << 4;
        re_tcp.sum = 0;

        let mut sum = ip.dst.to_be().to_be();
        sum += ip.src.to_be().to_be();
        sum += (ip.pro as u16).to_be() as u32;
        sum += ((data.len() + size_of::<TCPHeader>()) as u16).to_be() as u32;

        let mut check_sum = sum;
        let start = &re_tcp as *const _ as usize;
        let end = start + size_of::<TCPHeader>();
        for p in (start..end).step_by(2) {
            check_sum += unsafe { *(p as *const u16) as u32 };
            if check_sum > 0xffff {
                check_sum = (check_sum & 0xFFFF) + (check_sum >> 16);
                check_sum = check_sum + (check_sum >> 16);
            }
        }
        let start = data.as_slice().as_ptr() as usize;
        let end = start + data.len();
        for p in (start..end).step_by(2) {
            check_sum += unsafe { *(p as *const u16) as u32 };
            if check_sum > 0xffff {
                check_sum = (check_sum & 0xFFFF) + (check_sum >> 16);
                check_sum = check_sum + (check_sum >> 16);
            }
        }
        if data.len() %2 != 0 {
            check_sum += *data.last().unwrap() as u32;
            check_sum = (check_sum & 0xFFFF) + (check_sum >> 16);
            check_sum = check_sum + (check_sum >> 16);
        }
        let ans = !check_sum as u16;

        re_tcp.sum = ans;

        let header_data = unsafe { &*(&re_tcp as *const TCPHeader as *const [u8; 20]) };
        let header_data = header_data.to_vec();
        let mut total_data = header_data;
        total_data.extend(data);

        Net::send_ip(eth, ip, total_data);
    }
}
