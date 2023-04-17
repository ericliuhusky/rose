use core::mem::size_of;

use crate::{IPv4, MacAddress};

#[derive(PartialEq, Eq)]
pub enum EthType {
    IP,
    ARP,
}

#[derive(Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct Ip {
    pub(crate) vhl: u8,    // version << 4 | header length >> 2
    pub(crate) tos: u8,    // type of service
    pub(crate) len: u16,   // total length, packet length
    pub(crate) id: u16,    // identification, can combine all packets
    pub(crate) off: u16,   // fragment offset field, packet from
    pub(crate) ttl: u8,    // time to live
    pub(crate) pro: u8,    // protocol，TCP(6)、UDP(17)
    pub(crate) sum: u16,   // checksum,
    pub(crate) src: u32,   // souce ip
    pub(crate) dst: u32    // destination ip
}

#[allow(dead_code)]
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct UDP {
    pub(crate) sport: u16, // souce port
    pub(crate) dport: u16, // destination port
    pub(crate) ulen: u16,  // length, including udp header, not including IP header
    pub(crate) sum: u16    // checksum
}

bitflags! {
    // #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Copy)]
pub struct TCP {
    pub(crate) sport: u16, // souce port
    pub(crate) dport: u16, // destination port
    pub(crate) seq: u32, // sequence number
    pub(crate) ack: u32, // acknowledgement number
    pub(crate) offset: u8, // offset, first 4 bytes are tcp header length
    pub(crate) flags: TcpFlags, // flags, last 6 are flags(U, A, P, R, S, F)
    pub(crate) win: u16,    // window size
    pub(crate) sum: u16,    // checksum
    pub(crate) urg: u16,    // urgent pointer
}

pub(crate) const ETH_LEN: usize = size_of::<Eth>();
pub(crate) const IP_LEN:  usize = size_of::<Ip>();
pub(crate) const UDP_LEN: usize = size_of::<UDP>();
pub(crate) const TCP_LEN: usize = size_of::<TCP>();

/*
arp request and reply data
------------------------------ hexdump -------------------------------
ff ff ff ff ff ff 52 55 0a 00 02 02 08 06 00 01       ......RU........
08 00 06 04 00 01 52 55 0a 00 02 02 0a 00 02 02       ......RU........
00 00 00 00 00 00 0a 00 02 0f                         ..........                  
---------------------------- hexdump end -----------------------------

------------------------------ hexdump -------------------------------
ff ff ff ff ff ff 52 54 00 12 34 56 08 06 00 01       ......RT..4V....
08 00 06 04 00 02 52 54 00 12 34 56 0f 02 00 0a       ......RT..4V....
52 55 0a 00 02 02 0a 00 02 02                         RU........                  
---------------------------- hexdump end -----------------------------


the data
------------------------------ hexdump -------------------------------
52 54 00 12 34 56 52 55 0a 00 02 02 08 00 45 00       RT..4VRU......E.
00 2b 00 03 00 00 40 11 62 af 0a 00 02 02 0a 00       .+....@.b.......
02 0f d8 67 07 d0 00 17 35 21 74 68 69 73 20 69       ...g....5!this i
73 20 61 20 70 69 6e 67 21                            s a ping!                     
---------------------------- hexdump end -----------------------------
upd data
------------------------------ hexdump -------------------------------
74 68 69 73 20 69 73 20 61 20 70 69 6e 67 21          this is a ping!   
---------------------------- hexdump end -----------------------------

*/