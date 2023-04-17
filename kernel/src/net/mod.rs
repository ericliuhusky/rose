pub mod port_table;
pub mod tcp;
pub mod udp;

use core::mem::size_of;
use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::vec;
use alloc::vec::Vec;
use lose_net_stack::{packets::tcp::TCPPacket, Eth, EthType, Ip, UDP, IPProtocal};
pub use lose_net_stack::IPv4;
use lose_net_stack::{results::Packet, LoseStack, MacAddress, TcpFlags};

use self::tcp::TCP;

pub const LOCALHOST_IP: IPv4 = IPv4::new(10, 0, 2, 15);
pub const LOCALHOST_MAC: MacAddress = MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
const LOSE_NET_STACK: LoseStack = LoseStack::new(LOCALHOST_IP, LOCALHOST_MAC);

pub fn net_interrupt_handler() {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.analysis(&recv_buf[..len]);

    match packet {
        Packet::TCP(tcp_packet) => {
            let target = tcp_packet.source_ip;
            let lport = tcp_packet.dest_port;
            let rport = tcp_packet.source_port;
            let flags = tcp_packet.flags;

            if flags.contains(TcpFlags::F) {
                // tcp disconnected
                let reply_packet = tcp_packet.ack();
                NET_DEVICE.transmit(&reply_packet.build_data());

                let mut end_packet = reply_packet.ack();
                end_packet.flags |= TcpFlags::F;
                NET_DEVICE.transmit(&end_packet.build_data());
            }
        }
        _ => {}
    }
}

pub fn net_arp() {
    let (eth, arp) = Net::recv_arp();
    Net::send_arp(eth, arp);
}

pub fn net_accept(lport: u16) -> Option<TCP> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.analysis(&recv_buf[..len]);

    match packet {
        Packet::TCP(tcp_packet) => {
            let flags = tcp_packet.flags;

            if flags.contains(TcpFlags::S) {
                // if it has a port to accept, then response the request
                if lport == tcp_packet.dest_port {
                    let mut reply_packet = tcp_packet.ack();
                    reply_packet.flags = TcpFlags::S | TcpFlags::A;
                    NET_DEVICE.transmit(&reply_packet.build_data());

                    Some(TCP::new(
                        tcp_packet.source_ip,
                        tcp_packet.source_mac,
                        tcp_packet.source_port,
                        tcp_packet.dest_port,
                        tcp_packet.seq,
                        tcp_packet.ack,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None
    }
}

pub fn net_tcp_read(lport: u16, raddr: IPv4, rport: u16) -> Option<(Vec<u8>, u32, u32)> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.analysis(&recv_buf[..len]);

    match packet {
        Packet::TCP(tcp_packet) => {
            if tcp_packet.flags.contains(TcpFlags::A) {
                if tcp_packet.data_len == 0 {
                    return None;
                }
                if lport == tcp_packet.dest_port && raddr == tcp_packet.source_ip && rport == tcp_packet.source_port {
                    Some((tcp_packet.data.to_vec(), tcp_packet.seq, tcp_packet.ack))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None
    }
}

pub fn busy_wait_tcp_read(lport: u16, raddr: IPv4, rport: u16) -> (Vec<u8>, u32, u32) {
    loop {
        if let Some(data) = net_tcp_read(lport, raddr, rport) {
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

pub fn busy_wait_udp_read(lport: u16) -> (Vec<u8>, IPv4, MacAddress, u16) {
    let (eth, ip, udp, data) = TransPort::recv_udp(lport);
    let source_port = udp.sport.to_be();
    let source_ip = IPv4::from_u32(ip.src.to_be());
    let source_mac = MacAddress::new(eth.shost);
    (data, source_ip, source_mac, source_port)
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
    fn recv_arp() -> (Eth, Arp) {
        loop {
            let (eth, data) = Link::recv_eth();
            if eth.type_() == EthType::ARP {
                let arp = unsafe { &*(&data[..] as *const [u8] as *const Arp) };
                return (eth, *arp)
            }
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

    fn recv_ip() -> (Eth, Ip, Vec<u8>) {
        loop {
            let (eth, data) = Link::recv_eth();
            if eth.type_() == EthType::IP {
                let ip_len = size_of::<Ip>();
                let ip = unsafe { &*(&data[..ip_len] as *const [u8] as *const Ip) };
                let remain_data = &data[ip_len..];
                return (eth, *ip, remain_data.to_vec());
            }
        }
    }
}

struct TransPort;

impl TransPort {
    fn recv_udp(port: u16) -> (Eth, Ip, UDP, Vec<u8>) {
        loop {
            let (eth, ip, data) = Net::recv_ip();
            if ip.protocol() == IPProtocal::UDP {
                let udp_len = size_of::<UDP>();
                let udp = unsafe { &*(&data[..udp_len] as *const [u8] as *const UDP) };
                if udp.dport.to_be() == port {
                    let remain_data = &data[udp_len..];
                    return (eth, ip, *udp, remain_data.to_vec());
                }
            }
        }
    }
}
