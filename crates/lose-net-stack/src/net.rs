#[derive(PartialEq, Eq)]
pub enum EthType {
    IP,
    ARP,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
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
