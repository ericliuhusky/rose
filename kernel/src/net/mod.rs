pub mod port_table;
pub mod tcp;
pub mod udp;

use core::mem::size_of;
use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::vec;
use alloc::vec::Vec;
use core::mem::transmute;
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
    if let Some((tcp, data)) = TransPort::recv_tcp(lport) {    
        if tcp.tcp.flags.contains(TcpFlags::S) {
            let mut re_tcp = tcp.clone();
            re_tcp.tcp = tcp.tcp.ack();
            TransPort::send_tcp(re_tcp, vec![]);
    
            Some(TCP::new(
                tcp.tcp.dport.to_be(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn net_tcp_read(lport: u16) -> Option<(TCPPacket, Vec<u8>)> {
    if let Some((tcp, data)) = TransPort::recv_tcp(lport) {
        if tcp.tcp.flags.contains(TcpFlags::A) {
            if data.len() == 0 {
                return None;
            }
            Some((tcp, data))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn busy_wait_tcp_read(lport: u16) -> (TCPPacket, Vec<u8>) {
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

#[derive(Clone, Default)]
#[repr(packed)]
struct UDPPacketHeader {
    eth: Eth,
    ip: Ip,
    udp: UDPHeader,
}

#[derive(Clone, Default)]
struct UDPPacket {
    header: UDPPacketHeader,
    data: Vec<u8>,
}

#[derive(Clone, Default)]
#[repr(packed)]
pub struct TCPPacket {
    eth: Eth,
    ip: Ip,
    tcp: TCPHeader,
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
    fn recv_udp(port: u16) -> Option<UDPPacket> {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let udp_ptr = data.as_ptr() as *const UDPPacketHeader;
        let udp = unsafe { core::ptr::read(udp_ptr) };
        let data_ptr = unsafe { udp_ptr.offset(1) } as *const u8;
        let data_len = len - size_of::<UDPHeader>() - size_of::<Ip>() - size_of::<Eth>();
        let data = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };

        let udp = UDPPacket { header: udp, data: data.to_vec() };

        if udp.header.eth.type_() == EthType::IP && udp.header.ip.protocol() == IPProtocal::UDP && udp.header.udp.dport.to_be() == port {
            Some(udp)
        } else {
            None
        }
    }

    fn send_udp(udp: UDPPacket) {
        let mut re_udp = udp.header.clone();
        let data_len = udp.data.len();
        let len = data_len + size_of::<UDPHeader>();
        re_udp.udp.sport = udp.header.udp.dport;
        re_udp.udp.dport = udp.header.udp.sport;
        re_udp.udp.sum = 0;
        re_udp.udp.ulen = (len as u16).to_be();

        let len = data_len + size_of::<Ip>() + size_of::<UDPHeader>();
        re_udp.ip.src = udp.header.ip.dst;
        re_udp.ip.dst = udp.header.ip.src;
        re_udp.ip.len = (len as u16).to_be();
        re_udp.ip.sum = 0;
        re_udp.ip.sum = check_sum(&re_udp.ip as *const Ip as *const u8, size_of::<Ip>(), 0);

        re_udp.eth.dhost = udp.header.eth.shost;
        re_udp.eth.shost = LOCALHOST_MAC.to_bytes();

        let data: [u8; size_of::<UDPPacketHeader>()] = unsafe { transmute(re_udp) };
        let mut data = data.to_vec();
        data.extend(udp.data);

        NET_DEVICE.transmit(&data);
    }

    fn recv_tcp(port: u16) -> Option<(TCPPacket, Vec<u8>)> {
        let mut recv_buf = vec![0u8; 1024];
        let len = NET_DEVICE.receive(&mut recv_buf);
        let data = &recv_buf[..len];

        let tcp_ptr = data.as_ptr() as *const TCPPacket;
        let tcp = unsafe { core::ptr::read(tcp_ptr) };
        let data_ptr = unsafe { tcp_ptr.offset(1) } as *const u8;
        let data_len = len - size_of::<TCPHeader>() - size_of::<Ip>() - size_of::<Eth>();
        let data = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };

        if tcp.eth.type_() == EthType::IP && tcp.ip.protocol() == IPProtocal::TCP && tcp.tcp.dport.to_be() == port {
            Some((tcp, data.to_vec()))
        } else {
            None
        }
    }

    fn send_tcp(tcp: TCPPacket, data: Vec<u8>) {
        let mut re_tcp = tcp.clone();
        re_tcp.tcp.sport = tcp.tcp.dport;
        re_tcp.tcp.dport = tcp.tcp.sport;
        re_tcp.tcp.offset = 5 << 4;
        re_tcp.tcp.sum = 0;

        let mut sum = re_tcp.ip.dst.to_be().to_be();
        sum += re_tcp.ip.src.to_be().to_be();
        sum += (re_tcp.ip.pro as u16).to_be() as u32;
        sum += ((data.len() + size_of::<TCPHeader>()) as u16).to_be() as u32;

        let re_tcp_data: [u8; size_of::<TCPHeader>()] = unsafe { transmute(re_tcp.tcp) };
        let mut re_tcp_data = re_tcp_data.to_vec();
        re_tcp_data.extend(data.clone());

        let ans = check_sum(re_tcp_data.as_slice().as_ptr(), data.len() + size_of::<TCPHeader>(), sum);

        re_tcp.tcp.sum = ans;


        let len = data.len() + size_of::<Ip>() + size_of::<TCPHeader>();
        re_tcp.ip.src = tcp.ip.dst;
        re_tcp.ip.dst = tcp.ip.src;
        re_tcp.ip.len = (len as u16).to_be();
        re_tcp.ip.sum = 0;
        re_tcp.ip.sum = check_sum(&re_tcp.ip as *const Ip as *const u8, size_of::<Ip>(), 0);

        re_tcp.eth.dhost = tcp.eth.shost;
        re_tcp.eth.shost = LOCALHOST_MAC.to_bytes();

        let header_data: [u8; size_of::<TCPPacket>()] = unsafe { transmute(re_tcp) };
        let mut total_data = header_data.to_vec();
        total_data.extend(data);

        NET_DEVICE.transmit(&total_data);
    }
}


fn check_sum(ptr: *const u8, mut len: usize, mut sum: u32) -> u16 {
    let mut ptr = ptr as *const u16;

    while len > 1 {
        sum += unsafe { *ptr } as u32;
        unsafe { ptr = ptr.offset(1); }
        len -= 2;
    }

    if len == 1 {
        sum += unsafe { *(ptr as *const u8) } as u32;
    }

    fn fold(mut sum: u32) -> u16 {
        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }
        !sum as u16
    }
    fold(sum)
}
