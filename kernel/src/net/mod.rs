pub mod port_table;
pub mod tcp;
pub mod udp;

use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::{rc::Rc, vec};
use alloc::vec::Vec;
use lose_net_stack::packets::arp::ArpPacket;
use lose_net_stack::packets::tcp::TCPPacket;
use core::cell::RefCell;
pub use lose_net_stack::IPv4;
use lose_net_stack::{results::Packet, LoseStack, MacAddress, TcpFlags};

use self::tcp::TCP;

lazy_static::lazy_static! {
    pub static ref LOCALHOST_IP: IPv4 = IPv4::new(10, 0, 2, 15);
}
lazy_static::lazy_static! {
    pub static ref LOCALHOST_MAC: MacAddress = MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
}

pub struct NetStack(RefCell<LoseStack>);

impl NetStack {
    pub fn new() -> Self {
        unsafe {
            NetStack(RefCell::new(LoseStack::new(
                *LOCALHOST_IP,
                *LOCALHOST_MAC,
            )))
        }
    }
}

lazy_static::lazy_static! {
    static ref LOSE_NET_STACK: Rc<NetStack> = Rc::new(NetStack::new());
}

pub fn net_interrupt_handler() {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

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
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

    match packet {
        Packet::ARP(arp_packet) => {
            let lose_stack = LOSE_NET_STACK.0.borrow_mut();
            let reply_packet = arp_packet
                .reply_packet(lose_stack.ip, lose_stack.mac)
                .expect("can't build reply");
            let reply_data = reply_packet.build_data();
            NET_DEVICE.transmit(&reply_data)
        }
        _ => {}
    }
}

pub fn net_arp_request(raddr: IPv4) {
    let net_stack = LOSE_NET_STACK.0.borrow_mut();
    let arp_packet = ArpPacket::new(net_stack.ip, net_stack.mac, raddr, MacAddress::new([0; 6]), lose_net_stack::packets::arp::ArpType::Request);
    NET_DEVICE.transmit(&arp_packet.build_data());

    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = net_stack.analysis(&recv_buf[..len]);

    match packet {
        Packet::ARP(arp_packet) => {
            
        }
        _ => {}
    }
}

pub fn net_connect(ip: IPv4, port: u16) -> Option<TCP> {
    let net_stack = LOSE_NET_STACK.0.borrow_mut();
    // TODO: 自动分配端口号
    let tcp_packet = TCPPacket {
        source_ip: net_stack.ip,
        source_mac: net_stack.mac,
        source_port: 5000,
        dest_ip: ip,
        dest_mac: MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        dest_port: port,
        data_len: 0,
        seq: 0,
        ack: 0,
        flags: TcpFlags::S,
        win: 65535,
        urg: 0,
        data: &[],
    };
    NET_DEVICE.transmit(&tcp_packet.build_data());


    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = net_stack.analysis(&recv_buf[..len]);

    match packet {
        Packet::TCP(tcp_packet) => {
            println!("{} {}", tcp_packet.seq, tcp_packet.ack);

            Some(TCP::new(
                tcp_packet.source_ip,
                tcp_packet.source_mac,
                tcp_packet.source_port,
                tcp_packet.dest_port,
                tcp_packet.seq,
                tcp_packet.ack,
            ))
        }
        _ => None
    }
}

pub fn net_accept(lport: u16) -> Option<TCP> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

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

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

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

pub fn net_udp_read(lport: u16) -> Option<(Vec<u8>, IPv4, MacAddress, u16)> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

    match packet {
        Packet::UDP(udp_packet) => {
            if lport == udp_packet.dest_port {
                Some((udp_packet.data.to_vec(), udp_packet.source_ip, udp_packet.source_mac, udp_packet.source_port))
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
    loop {
        if let Some(data) = net_udp_read(lport) {
            return data;
        }
    }
}
