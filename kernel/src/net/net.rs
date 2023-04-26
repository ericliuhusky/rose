use super::check_sum::check_sum;
use super::{
    ARPPacket, ArpType, EthType, EthernetHeader, IPHeader, IPProtocal, TCPHeader, TCPPacket,
    TCPPacketHeader, UDPHeader, UDPPacket, UDPPacketHeader, LOCALHOST_IP, LOCALHOST_MAC,
};
use crate::drivers::virtio_net::NET_DEVICE;
use alloc::vec;
use core::mem::{size_of, transmute};

pub fn recv_arp() -> Option<ARPPacket> {
    let mut recv_buf = vec![0u8; 1024];
    let len = NET_DEVICE.receive(&mut recv_buf);
    let data = &recv_buf[..len];

    let arp_ptr = data.as_ptr() as *const ARPPacket;
    let arp = unsafe { core::ptr::read(arp_ptr) };

    if arp.eth.type_() == EthType::ARP {
        Some(arp)
    } else {
        None
    }
}

pub fn send_arp(arp: ARPPacket) {
    let mut re_arp = arp.clone();
    re_arp.arp.spa = LOCALHOST_IP.to_be();
    re_arp.arp.sha = LOCALHOST_MAC;
    re_arp.arp.tpa = arp.arp.spa;
    re_arp.arp.tha = arp.arp.sha;
    re_arp.arp.set_type(ArpType::Reply);

    re_arp.eth.dhost = arp.eth.shost;
    re_arp.eth.shost = LOCALHOST_MAC;

    let data: [u8; size_of::<ARPPacket>()] = unsafe { transmute(re_arp) };

    NET_DEVICE.transmit(&data);
}

pub fn recv_udp(port: u16) -> Option<UDPPacket> {
    let mut recv_buf = vec![0u8; 1024];
    let len = NET_DEVICE.receive(&mut recv_buf);
    let data = &recv_buf[..len];

    let udp_ptr = data.as_ptr() as *const UDPPacketHeader;
    let udp = unsafe { core::ptr::read(udp_ptr) };
    let data_ptr = unsafe { udp_ptr.offset(1) } as *const u8;
    let data_len =
        len - size_of::<UDPHeader>() - size_of::<IPHeader>() - size_of::<EthernetHeader>();
    let data = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };

    let udp = UDPPacket {
        header: udp,
        data: data.to_vec(),
    };

    if udp.header.eth.type_() == EthType::IP
        && udp.header.ip.protocol() == IPProtocal::UDP
        && udp.header.udp.dport.to_be() == port
    {
        Some(udp)
    } else {
        None
    }
}

pub fn send_udp(udp: UDPPacket) {
    let mut re_udp = udp.header.clone();
    let data_len = udp.data.len();
    let len = data_len + size_of::<UDPHeader>();
    re_udp.udp.sport = udp.header.udp.dport;
    re_udp.udp.dport = udp.header.udp.sport;
    re_udp.udp.sum = 0;
    re_udp.udp.ulen = (len as u16).to_be();

    let len = data_len + size_of::<IPHeader>() + size_of::<UDPHeader>();
    re_udp.ip.src = udp.header.ip.dst;
    re_udp.ip.dst = udp.header.ip.src;
    re_udp.ip.len = (len as u16).to_be();
    re_udp.ip.sum = 0;
    re_udp.ip.sum = check_sum(
        &re_udp.ip as *const IPHeader as *const u8,
        size_of::<IPHeader>(),
        0,
    );

    re_udp.eth.dhost = udp.header.eth.shost;
    re_udp.eth.shost = udp.header.eth.dhost;

    let data: [u8; size_of::<UDPPacketHeader>()] = unsafe { transmute(re_udp) };
    let mut data = data.to_vec();
    data.extend(udp.data);

    NET_DEVICE.transmit(&data);
}

pub fn recv_tcp(port: u16) -> Option<TCPPacket> {
    let mut recv_buf = vec![0u8; 1024];
    let len = NET_DEVICE.receive(&mut recv_buf);
    let data = &recv_buf[..len];

    let tcp_ptr = data.as_ptr() as *const TCPPacketHeader;
    let tcp = unsafe { core::ptr::read(tcp_ptr) };
    let data_ptr = unsafe { tcp_ptr.offset(1) } as *const u8;
    let data_len =
        len - size_of::<TCPHeader>() - size_of::<IPHeader>() - size_of::<EthernetHeader>();
    let data = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };

    let tcp = TCPPacket {
        header: tcp,
        data: data.to_vec(),
    };

    if tcp.header.eth.type_() == EthType::IP
        && tcp.header.ip.protocol() == IPProtocal::TCP
        && tcp.header.tcp.dport.to_be() == port
    {
        Some(tcp)
    } else {
        None
    }
}

pub fn send_tcp(tcp: TCPPacket) {
    let mut re_tcp = tcp.header.clone();
    re_tcp.tcp.sport = tcp.header.tcp.dport;
    re_tcp.tcp.dport = tcp.header.tcp.sport;
    re_tcp.tcp.offset = 5 << 4;
    re_tcp.tcp.sum = 0;

    let mut sum = re_tcp.ip.dst.to_be().to_be();
    sum += re_tcp.ip.src.to_be().to_be();
    sum += (re_tcp.ip.pro as u16).to_be() as u32;
    sum += ((tcp.data.len() + size_of::<TCPHeader>()) as u16).to_be() as u32;

    let re_tcp_data: [u8; size_of::<TCPHeader>()] = unsafe { transmute(re_tcp.tcp) };
    let mut re_tcp_data = re_tcp_data.to_vec();
    re_tcp_data.extend(tcp.data.clone());

    let ans = check_sum(
        re_tcp_data.as_slice().as_ptr(),
        tcp.data.len() + size_of::<TCPHeader>(),
        sum,
    );

    re_tcp.tcp.sum = ans;

    let len = tcp.data.len() + size_of::<IPHeader>() + size_of::<TCPHeader>();
    re_tcp.ip.src = tcp.header.ip.dst;
    re_tcp.ip.dst = tcp.header.ip.src;
    re_tcp.ip.len = (len as u16).to_be();
    re_tcp.ip.sum = 0;
    re_tcp.ip.sum = check_sum(
        &re_tcp.ip as *const IPHeader as *const u8,
        size_of::<IPHeader>(),
        0,
    );

    re_tcp.eth.dhost = tcp.header.eth.shost;
    re_tcp.eth.shost = tcp.header.eth.dhost;

    let header_data: [u8; size_of::<TCPPacketHeader>()] = unsafe { transmute(re_tcp) };
    let mut total_data = header_data.to_vec();
    total_data.extend(tcp.data);

    NET_DEVICE.transmit(&total_data);
}
