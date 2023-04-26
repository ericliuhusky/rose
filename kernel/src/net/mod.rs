pub mod port_table;
pub mod tcp;
pub mod udp;
mod net;
mod check_sum;

use alloc::vec;
use alloc::vec::Vec;
use self::net::{recv_arp, send_arp, recv_tcp, send_tcp};
use self::tcp::TCP;

pub const LOCALHOST_IP: IPv4 = IPv4::new(10, 0, 2, 15);
pub const LOCALHOST_MAC: MacAddress = MacAddress::new([1, 2, 3, 4, 5, 6]);


pub fn net_arp() {
    if let Some(arp) = recv_arp() {
        send_arp(arp);
    }
}

pub fn net_accept(lport: u16) -> Option<TCP> {
    if let Some(tcp) = recv_tcp(lport) {    
        if tcp.header.tcp.flags.contains(TcpFlags::S) {
            let mut re_tcp = tcp.header.clone();
            re_tcp.tcp = tcp.header.tcp.ack();
            send_tcp(TCPPacket { header: re_tcp, data: vec![] });
    
            Some(TCP::new(
                tcp.header.tcp.dport.to_be(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn net_tcp_read(lport: u16) -> Option<TCPPacket> {
    if let Some(tcp) = recv_tcp(lport) {
        if tcp.header.tcp.flags.contains(TcpFlags::A) {
            if tcp.data.len() == 0 {
                return None;
            }
            Some(tcp)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn busy_wait_tcp_read(lport: u16) -> TCPPacket {
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
pub struct EthernetHeader {
    pub dhost: [u8; 6], // destination host
    pub shost: [u8; 6], // source host
    pub rtype: u16      // packet type, arp or ip
}

impl EthernetHeader {
    pub fn type_(&self) -> EthType {
        match self.rtype.to_be() {
            0x800 => EthType::IP,
            0x806 => EthType::ARP,
            _ => panic!()
        }
    }
}


#[allow(dead_code)]
#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IPHeader {
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

impl IPHeader {
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
    Reply,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ARPHeader {
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

impl ARPHeader {
    pub fn set_type(&mut self, type_: ArpType) {
        let op: u16 = match type_ {
            ArpType::Reply => 2,
        };
        self.op = op.to_be();
    }
}

#[derive(Clone)]
#[repr(packed)]
pub struct ARPPacket {
    eth: EthernetHeader,
    arp: ARPHeader,
}

#[derive(Clone, Default)]
#[repr(packed)]
struct UDPPacketHeader {
    eth: EthernetHeader,
    ip: IPHeader,
    udp: UDPHeader,
}

#[derive(Clone, Default)]
pub struct UDPPacket {
    header: UDPPacketHeader,
    data: Vec<u8>,
}

#[derive(Clone, Default)]
#[repr(packed)]
struct TCPPacketHeader {
    eth: EthernetHeader,
    ip: IPHeader,
    tcp: TCPHeader,
}

#[derive(Clone, Default)]
pub struct TCPPacket {
    header: TCPPacketHeader,
    data: Vec<u8>,
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
