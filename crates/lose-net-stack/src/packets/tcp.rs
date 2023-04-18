use alloc::vec::Vec;

use crate::consts::{IP_HEADER_VHL, IP_PROTOCAL_TCP};
use crate::utils::{UnsafeRefIter, check_sum};
use crate::IPv4;
use crate::MacAddress;
use crate::net::{TCP_LEN, IP_LEN, ETH_LEN, Eth, Ip, TCPHeader, TcpFlags, EthType};

#[derive(Clone)]
pub struct TCPPacket {
    pub source_ip: IPv4,
    pub source_mac: MacAddress,
    pub source_port: u16,
    pub dest_ip: IPv4,
    pub dest_mac: MacAddress,
    pub dest_port: u16,
    pub data_len: usize,

    pub seq: u32,           // sequence number
    pub ack: u32,           // acknowledgement number
    pub flags: TcpFlags,    // flags, last 6 are flags(U, A, P, R, S, F)
    pub win: u16,           // window size
    pub urg: u16,           // urgent pointer
    pub data: Vec<u8>      // data buffer
}
