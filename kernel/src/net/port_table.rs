use alloc::vec::Vec;
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use mutrc::MutRc;

use crate::fs::File;
use crate::task::current_task;
use crate::task::task::Task;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
    pub receivable: bool,
}

lazy_static! {
    static ref LISTEN_TABLE: RefCell<Vec<MutRc<Port>>> = RefCell::new(Vec::new());
}

pub fn listen(port: u16) -> MutRc<Port> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();

    let listen_port = MutRc::new(Port {
        port,
        receivable: false,
    });

    listen_table.push(listen_port.clone());
    listen_port
}

// can accept request
pub fn accept(mut port: MutRc<Port>) {
    port.receivable = true;
}

pub fn port_acceptable(port: MutRc<Port>) -> bool {
    port.receivable
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<()> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let listen_port = listen_table.iter_mut().find(|p| p.port == port && p.receivable == true);
    if let Some(listen_port) = listen_port {
        listen_port.receivable = false;

        accept_connection(port, tcp_packet);
        Some(())
    } else {
        None
    }
}

pub fn accept_connection(_port: u16, tcp_packet: &TCPPacket) {
    let mut task = current_task();
    let mut process = task.process.upgrade().unwrap();

    let tcp_socket = TCP::new(
        tcp_packet.source_ip,
        tcp_packet.dest_port,
        tcp_packet.source_port,
        tcp_packet.seq,
        tcp_packet.ack,
    );

    let fd = process.fd_table.insert(MutRc::new(tcp_socket));

    task.cx.x[10] = fd;
}

impl File for Port {
    fn read(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }

    fn write(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        unimplemented!()
    }
}
