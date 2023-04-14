pub mod port_table;
pub mod tcp;
pub mod udp;

use crate::{
    drivers::virtio_net::NET_DEVICE,
};
use alloc::{rc::Rc, vec};
use alloc::vec::Vec;
use core::cell::RefCell;
pub use lose_net_stack::IPv4;
use lose_net_stack::{results::Packet, LoseStack, MacAddress, TcpFlags};

use self::{port_table::check_accept, tcp::TCP};

pub struct NetStack(RefCell<LoseStack>);

impl NetStack {
    pub fn new() -> Self {
        unsafe {
            NetStack(RefCell::new(LoseStack::new(
                IPv4::new(10, 0, 2, 15),
                MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
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

pub fn net_accept() -> Option<TCP> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

    match packet {
        Packet::TCP(tcp_packet) => {
            let lport = tcp_packet.dest_port;
            let flags = tcp_packet.flags;

            if flags.contains(TcpFlags::S) {
                // if it has a port to accept, then response the request
                let tcp_socket = check_accept(lport, &tcp_packet);
                if tcp_socket.is_some() {
                    let mut reply_packet = tcp_packet.ack();
                    reply_packet.flags = TcpFlags::S | TcpFlags::A;
                    NET_DEVICE.transmit(&reply_packet.build_data());
                }
                tcp_socket
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

pub fn net_udp_read(lport: u16) -> Option<(Vec<u8>, IPv4, u16)> {
    let mut recv_buf = vec![0u8; 1024];

    let len = NET_DEVICE.receive(&mut recv_buf);

    let packet = LOSE_NET_STACK.0.borrow_mut().analysis(&recv_buf[..len]);

    match packet {
        Packet::UDP(udp_packet) => {
            if lport == udp_packet.dest_port {
                Some((udp_packet.data.to_vec(), udp_packet.source_ip, udp_packet.source_port))
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

pub fn busy_wait_accept() -> TCP {
    loop {
        if let Some(socket) = net_accept() {
            return socket;
        }
    }
}

pub fn busy_wait_udp_read(lport: u16) -> (Vec<u8>, IPv4, u16) {
    loop {
        if let Some(data) = net_udp_read(lport) {
            return data;
        }
    }
}

#[allow(unused)]
pub fn hexdump(data: &[u8]) {
    const PRELAND_WIDTH: usize = 70;
    println!("[kernel] {:-^1$}", " hexdump ", PRELAND_WIDTH);
    for offset in (0..data.len()).step_by(16) {
        print!("[kernel] ");
        for i in 0..16 {
            if offset + i < data.len() {
                print!("{:02x} ", data[offset + i]);
            } else {
                print!("{:02} ", "");
            }
        }

        print!("{:>6}", ' ');

        for i in 0..16 {
            if offset + i < data.len() {
                let c = data[offset + i];
                if c >= 0x20 && c <= 0x7e {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            } else {
                print!("{:02} ", "");
            }
        }

        println!("");
    }
    println!("[kernel] {:-^1$}", " hexdump end ", PRELAND_WIDTH);
}
