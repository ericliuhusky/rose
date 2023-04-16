use core::mem::size_of;
use alloc::vec::Vec;

use crate::{IPv4, MacAddress, utils::UnsafeRefIter, consts::{ETH_RTYPE_ARP, BROADCAST_MAC, ARP_HRD_ETHER, ARP_ETHADDR_LEN}};

pub enum EthType {
    IP,
    ARP,
}

#[repr(C)]
pub struct Eth {
    pub(crate) dhost: [u8; 6], // destination host
    pub(crate) shost: [u8; 6], // source host
    pub(crate) rtype: u16      // packet type, arp or ip
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


#[derive(Debug, Clone, Copy)]
pub enum ArpType {
    Request,
    Reply,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Arp {
    pub(crate) httype: u16, // Hardware type
    pub(crate) pttype: u16, // Protocol type, For IPv4, this has the value 0x0800.
    pub(crate) hlen: u8,    // Hardware length: Ethernet address length is 6.
    pub(crate) plen: u8,    // Protocol length: IPv4 address length is 4.
    op: u16,     // Operation: 1 for request, 2 for reply.
    pub(crate) sha: [u8; 6],// Sender hardware address
    pub(crate) spa: u32,    // Sender protocol address
    pub(crate) tha: [u8; 6],// Target hardware address
    pub(crate) tpa: u32     // Target protocol address
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

    pub fn reply_packet(&self, local_ip: IPv4, local_mac: MacAddress) -> Self {
        match self.type_() {
            ArpType::Request => {
                let mut reply_packet = Self::default();
                reply_packet.set_src_ip(local_ip);
                reply_packet.set_src_mac(local_mac);
                reply_packet.set_dst_ip(self.src_ip());
                reply_packet.set_dst_mac(self.src_mac());
                reply_packet.set_type(ArpType::Reply);
                reply_packet
            }
            _ => panic!()
        }
    }

    pub fn build_data(&self) -> Vec<u8> {
        let data = vec![0u8; ARP_LEN + ETH_LEN];

        let mut data_ptr_iter = UnsafeRefIter::new(&data);

        // let mut eth_header = unsafe{(data.as_ptr() as usize as *mut Eth).as_mut()}.unwrap();
        let mut eth_header = unsafe{ data_ptr_iter.next_mut::<Eth>() }.unwrap();
        eth_header.rtype = ETH_RTYPE_ARP.to_be();
        eth_header.dhost = BROADCAST_MAC;
        eth_header.shost = self.src_mac().to_bytes();

        // let mut arp_header = unsafe{((data.as_ptr() as usize + size_of::<Eth>()) as *mut Arp).as_mut()}.unwrap();
        let mut arp_header = unsafe { data_ptr_iter.next_mut::<Arp>() }.unwrap();
        arp_header.httype = ARP_HRD_ETHER.to_be();
        arp_header.pttype = ETH_RTYPE_ARP.to_be();
        arp_header.hlen = ARP_ETHADDR_LEN as u8;    // mac address len
        arp_header.plen = 4;    // ipv4
        arp_header.op = self.op;
        
        arp_header.sha = self.sha;
        arp_header.spa = self.spa;
    
        arp_header.tha = self.tha;
        arp_header.tpa = self.tpa;
        data
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
pub(crate) const ARP_LEN: usize = size_of::<Arp>();
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