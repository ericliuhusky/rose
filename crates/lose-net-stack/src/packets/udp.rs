use alloc::vec::Vec;

use crate::consts::{IP_PROTOCAL_UDP, IP_HEADER_VHL};
use crate::net::{UDP, Eth, Ip, UDP_LEN, IP_LEN, ETH_LEN, EthType};
use crate::IPv4;
use crate::MacAddress;
use crate::utils::{UnsafeRefIter, check_sum};

#[derive(Clone)]
pub struct UDPPacket {
    pub source_ip: IPv4,
    pub source_mac: MacAddress,
    pub source_port: u16,
    pub dest_ip: IPv4,
    pub dest_mac: MacAddress,
    pub dest_port: u16,
    pub data_len: usize,
    pub data: Vec<u8>
}

impl UDPPacket {
    pub fn new(source_ip: IPv4, source_mac: MacAddress, source_port: u16, 
        dest_ip: IPv4, dest_mac: MacAddress, dest_port: u16, 
        data_len: usize, data: Vec<u8>) -> Self {
        Self {
            source_ip,
            source_mac,
            source_port,
            dest_ip,
            dest_mac,
            dest_port,
            data_len,
            data
        }
    }

    pub fn build_data(&self) -> Vec<u8> {
        let data = vec![0u8; UDP_LEN + IP_LEN + ETH_LEN + self.data_len];

        // convert data ptr to the ref needed.
        let mut data_ptr_iter = UnsafeRefIter::new(&data);
        let eth_header = unsafe{data_ptr_iter.next_mut::<Eth>()}.unwrap();
        let ip_header = unsafe{data_ptr_iter.next_mut::<Ip>()}.unwrap();
        let udp_header = unsafe{data_ptr_iter.next_mut::<UDP>()}.unwrap();
        let udp_data = unsafe {data_ptr_iter.get_curr_arr_mut()};


        eth_header.set_type(EthType::IP);
        eth_header.shost = self.source_mac.to_bytes();
        eth_header.dhost = self.dest_mac.to_bytes();
        
        ip_header.pro = IP_PROTOCAL_UDP.to_be();
        ip_header.off = 0;
        ip_header.src = self.source_ip.to_u32().to_be();
        ip_header.dst = self.dest_ip.to_u32().to_be();
        ip_header.tos = 0; // type of service, use 0 as default
        ip_header.id  = 0; // packet identified, use 0 as default
        ip_header.ttl = 100; // packet ttl, use 32 as default
        ip_header.vhl = IP_HEADER_VHL; // version << 4 | header length >> 2
        ip_header.len = ((self.data_len + UDP_LEN + IP_LEN) as u16).to_be(); // toal len
        ip_header.sum = check_sum(ip_header as *mut Ip as *mut u8, IP_LEN as _, 0); // checksum

        udp_header.sport = self.source_port.to_be();
        udp_header.dport = self.dest_port.to_be();
        udp_header.sum   = 0; // udp checksum   zero means no checksum is provided.
        udp_header.ulen  = ((self.data_len + UDP_LEN) as u16).to_be();

        udp_data.copy_from_slice(&self.data);

        data
    }
}