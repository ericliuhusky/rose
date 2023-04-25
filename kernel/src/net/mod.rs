pub mod port_table;
pub mod tcp;
pub mod udp;

use core::mem::size_of;
use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::vec;
use alloc::vec::Vec;
use core_ext::UInt;

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
    if let Some(arp) = Net::recv_arp() {
        Net::send_arp(arp);
    }
}

pub fn net_accept(lport: u16) -> Option<TCP> {
    if let Some((eth, ip, tcp, data)) = TransPort::recv_tcp(lport) {    
        if tcp.flags.contains(TcpFlags::S) {
            TransPort::send_tcp(eth, ip, tcp.ack(), vec![]);
    
            Some(TCP::new(
                tcp.dport.to_be(),
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

pub fn net_tcp_read(lport: u16) -> Option<(Eth, Ip, TCPHeader, Vec<u8>)> {
    if let Some((eth, ip, tcp, data)) = TransPort::recv_tcp(lport) {
        if tcp.flags.contains(TcpFlags::A) {
            if data.len() == 0 {
                return None;
            }
            Some((eth, ip, tcp, data))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn busy_wait_tcp_read(lport: u16) -> (Eth, Ip, TCPHeader, Vec<u8>) {
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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct IPv4(u32);

impl IPv4 {
    pub const fn new(a1: u8, a2: u8, a3: u8, a4: u8) -> Self {
        IPv4((a1 as u32) << 24 | (a2 as u32) << 16 | (a3 as u32) << 8 | (a4 as u32))
    }

    pub fn from_u32(ip: u32) -> Self {
        IPv4(ip)
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Default)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(addr: [u8; 6]) -> Self {
        MacAddress(addr)
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        self.0
    }
}


#[derive(PartialEq, Eq)]
pub enum EthType {
    IP,
    ARP,
}

#[derive(Clone, Copy, Default)]
#[repr(packed)]
pub struct Eth {
    pub dhost: [u8; 6], // destination host
    pub shost: [u8; 6], // source host
    pub rtype: u16      // packet type, arp or ip
}

impl Eth {
    pub fn type_(&self) -> EthType {
        match self.rtype.to_be() {
            0x800 => EthType::IP,
            0x806 => EthType::ARP,
            _ => panic!()
        }
    }

    pub fn set_type(&mut self, type_: EthType) {
        let type_: u16 = match type_ {
            EthType::IP => 0x800,
            EthType::ARP => 0x806,
        };
        self.rtype = type_.to_be();
    }
}


#[allow(dead_code)]
#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ip {
    pub vhl: u8,    // version << 4 | header length >> 2
    pub tos: u8,    // type of service
    pub len: u16,   // total length, packet length
    pub id: u16,    // identification, can combine all packets
    pub off: u16,   // fragment offset field, packet from
    pub ttl: u8,    // time to live
    pub pro: u8,    // protocol，TCP(6)、UDP(17)
    pub sum: u16,   // checksum,
    pub src: u32,   // souce ip
    pub dst: u32    // destination ip
}

#[derive(PartialEq, Eq)]
pub enum IPProtocal {
    TCP,
    UDP,
}

impl Ip {
    pub fn protocol(&self) -> IPProtocal {
        match self.pro {
            6 => IPProtocal::TCP,
            17 => IPProtocal::UDP,
            _ => panic!()
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
}

#[derive(Clone)]
#[repr(packed)]
struct ArpPacket {
    eth: Eth,
    arp: Arp,
}

#[derive(Clone)]
#[repr(packed)]
struct UDPPacket {
    eth: Eth,
    ip: Ip,
    udp: UDPHeader,
    data: [u8; 1024],
}

#[repr(packed)]
struct TCPPacket {
    eth: Eth,
    ip: Ip,
    tcp: TCPHeader,
    data: [u8; 1024],
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UDPHeader {
    pub sport: u16, // souce port
    pub dport: u16, // destination port
    pub ulen: u16,  // length, including udp header, not including IP header
    pub sum: u16    // checksum
}



bitflags! {
    // #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(Default)]
    pub struct TcpFlags: u8 {
        const NONE = 0;
        const F = 0b00000001;
        const S = 0b00000010;
        const R = 0b00000100;
        const P = 0b00001000;
        const A = 0b00010000;
        const U = 0b00100000;
    }
}

#[allow(dead_code)]
#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TCPHeader {
    pub sport: u16, // souce port
    pub dport: u16, // destination port
    pub seq: u32, // sequence number
    pub ack: u32, // acknowledgement number
    pub offset: u8, // offset, first 4 bytes are tcp header length
    pub flags: TcpFlags, // flags, last 6 are flags(U, A, P, R, S, F)
    pub win: u16,    // window size
    pub sum: u16,    // checksum
    pub urg: u16,    // urgent pointer
}

impl TCPHeader {
    pub fn ack(&self) -> Self {
        Self {
            sport: self.sport,
            dport: self.dport,
            seq: 0,
            ack: (self.seq.to_be() + 1).to_be(),
            offset: self.offset,
            flags: TcpFlags::S | TcpFlags::A,
            win: self.win,
            urg: self.urg,
            sum: 0,
        }
    }
}


struct Net;

impl Net {
    fn recv_arp() -> Option<ArpPacket> {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let arp_ptr = data.as_ptr() as *const ArpPacket;
        let arp = unsafe { core::ptr::read(arp_ptr) };

        if arp.eth.type_() == EthType::ARP {
            Some(arp)
        } else {
            None
        }
    }

    fn send_arp(arp: ArpPacket) {
        let mut re_arp = arp.clone();
        re_arp.arp.spa = LOCALHOST_IP.to_u32().to_be();
        re_arp.arp.sha = LOCALHOST_MAC.to_bytes();
        re_arp.arp.tpa = arp.arp.spa;
        re_arp.arp.tha = arp.arp.sha;
        re_arp.arp.set_type(ArpType::Reply);

        re_arp.eth.dhost = arp.eth.shost;
        re_arp.eth.shost = LOCALHOST_MAC.to_bytes();

        let data: [u8; size_of::<ArpPacket>()] = unsafe { transmute(re_arp) };

        NET_DEVICE.transmit(&data);
    }
}

struct TransPort;

impl TransPort {
    fn recv_udp(port: u16) -> Option<(UDPPacket, usize)> {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let udp_ptr = data.as_ptr() as *const UDPPacket;
        let udp = unsafe { core::ptr::read(udp_ptr) };
        let data_len = len - size_of::<UDPHeader>() - size_of::<Ip>() - size_of::<Eth>();

        if udp.eth.type_() == EthType::IP && udp.ip.protocol() == IPProtocal::UDP && udp.udp.dport.to_be() == port {
            Some((udp, data_len))
        } else {
            None
        }
    }

    fn send_udp(udp: UDPPacket, data_len: usize) {
        let mut re_udp = udp.clone();
        let len = data_len + size_of::<UDPHeader>();
        re_udp.udp.sport = udp.udp.dport;
        re_udp.udp.dport = udp.udp.sport;
        re_udp.udp.sum = 0;
        re_udp.udp.ulen = (len as u16).to_be();

        let len = data_len + size_of::<Ip>() + size_of::<UDPHeader>();
        re_udp.ip.src = udp.ip.dst;
        re_udp.ip.dst = udp.ip.src;
        re_udp.ip.len = (len as u16).to_be();
        re_udp.ip.sum = 0;
        re_udp.ip.sum = check_sum(&mut re_udp.ip as *mut Ip as *mut u8, size_of::<Ip>() as _, 0);

        re_udp.eth.dhost = udp.eth.shost;
        re_udp.eth.shost = LOCALHOST_MAC.to_bytes();

        let data: [u8; size_of::<UDPPacket>()] = unsafe { transmute(re_udp) };
        let data = &data[..(size_of::<Eth>() + size_of::<Ip>() + size_of::<UDPHeader>() + data_len)];

        NET_DEVICE.transmit(data);
    }

    fn recv_tcp(port: u16) -> Option<(Eth, Ip, TCPHeader, Vec<u8>)> {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let tcp_ptr = data.as_ptr() as *const TCPPacket;
        let tcp = unsafe { core::ptr::read(tcp_ptr) };
        let data = &tcp.data[..(len - size_of::<TCPHeader>() - size_of::<Ip>() - size_of::<Eth>())];


        if tcp.eth.type_() == EthType::IP && tcp.ip.protocol() == IPProtocal::TCP && tcp.tcp.dport.to_be() == port {
            let offset = ((tcp.tcp.offset >> 4 & 0xf) as usize - 5) * 4;
            let offset_data = &data[offset..];
            Some((tcp.eth, tcp.ip, tcp.tcp, offset_data.to_vec()))
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

        let mut ck_sum = sum;
        let start = &re_tcp as *const _ as usize;
        let end = start + size_of::<TCPHeader>();
        for p in (start..end).step_by(2) {
            ck_sum += unsafe { *(p as *const u16) as u32 };
            if ck_sum > 0xffff {
                ck_sum = (ck_sum & 0xFFFF) + (ck_sum >> 16);
                ck_sum = ck_sum + (ck_sum >> 16);
            }
        }
        let start = data.as_slice().as_ptr() as usize;
        let end = start + UInt(data.len()).align_to_lower(2);
        for p in (start..end).step_by(2) {
            ck_sum += unsafe { *(p as *const u16) as u32 };
            if ck_sum > 0xffff {
                ck_sum = (ck_sum & 0xFFFF) + (ck_sum >> 16);
                ck_sum = ck_sum + (ck_sum >> 16);
            }
        }
        if data.len() %2 != 0 {
            ck_sum += *data.last().unwrap() as u32;
            ck_sum = (ck_sum & 0xFFFF) + (ck_sum >> 16);
            ck_sum = ck_sum + (ck_sum >> 16);
        }
        let ans = !ck_sum as u16;

        re_tcp.sum = ans;


        let len = data.len() + size_of::<Ip>() + size_of::<TCPHeader>();
        let mut re_ip = ip;
        re_ip.src = ip.dst;
        re_ip.dst = ip.src;
        re_ip.len = (len as u16).to_be();
        re_ip.sum = 0;
        re_ip.sum = check_sum(&mut re_ip as *mut Ip as *mut u8, size_of::<Ip>() as _, 0);

        let mut re_eth = eth;
        re_eth.dhost = eth.shost;
        re_eth.shost = LOCALHOST_MAC.to_bytes();

        let header_data = headers_to_data(
            vec![
                Header::ETH(re_eth),
                Header::IP(re_ip),
                Header::TCP(re_tcp),
            ]
        );

        let mut total_data = header_data;
        total_data.extend(data);

        NET_DEVICE.transmit(&total_data);
    }
}

pub fn check_sum(addr:*mut u8, len:u32, sum: u32) -> u16 {
    let mut sum:u32 = sum;
    let mut nleft = len;
    let mut w = addr as *const u16;
    
     while nleft > 1 {
        sum += unsafe{ *w as u32 };
        w = (w as usize + 2) as *mut u16;
        nleft -= 2;

        if sum > 0xffff {
            sum = (sum & 0xFFFF) + (sum >> 16);
            sum = sum + (sum >> 16);
        }
     }

     if nleft == 1 {
        sum += unsafe { *(w as *const u8) as u32};
     }


     sum = (sum & 0xFFFF) + (sum >> 16);
     sum = sum + (sum >> 16);

     let answer:u16 = !sum as u16;

     answer
}


enum Header {
    ETH(Eth),
    ARP(Arp),
    IP(Ip),
    UDP(UDPHeader),
    TCP(TCPHeader)
}

use core::mem::transmute;

fn headers_to_data(headers: Vec<Header>) -> Vec<u8> {
    let mut data = Vec::new();
    for header in headers {
        match header {
            Header::ETH(eth) => {
                let header_data: [u8; size_of::<Eth>()] = unsafe { transmute(eth) };
                data.extend(header_data);
            }
            Header::ARP(arp) => {
                let header_data: [u8; size_of::<Arp>()] = unsafe { transmute(arp) };
                data.extend(header_data);
            }
            Header::IP(ip) => {
                let header_data: [u8; size_of::<Ip>()] = unsafe { transmute(ip) };
                data.extend(header_data);
            }
            Header::UDP(udp) => {
                let header_data: [u8; size_of::<UDPHeader>()] = unsafe { transmute(udp) };
                data.extend(header_data);
            }
            Header::TCP(tcp) => {
                let header_data: [u8; size_of::<TCPHeader>()] = unsafe { transmute(tcp) };
                data.extend(header_data);
            }
        }
    }
    data
}
