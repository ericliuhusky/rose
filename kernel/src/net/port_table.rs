use alloc::vec::Vec;
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use mutrc::MutRc;

use crate::fs::File;
use crate::task::id::IDAllocDict;
use crate::task::task::Task;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
    pub receivable: bool,
    pub schedule: Option<MutRc<Task>>,
}

lazy_static! {
    static ref LISTEN_TABLE: RefCell<IDAllocDict<Port>> = RefCell::new(IDAllocDict::new());
}

pub fn listen(port: u16) -> usize {
    let mut listen_table = LISTEN_TABLE.borrow_mut();

    let listen_port = Port {
        port,
        receivable: false,
        schedule: None,
    };

    listen_table.insert(listen_port)
}

// can accept request
pub fn accept(listen_index: usize, task: MutRc<Task>) {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let listen_port = listen_table.get_mut(listen_index);
    let listen_port = listen_port.unwrap();
    listen_port.receivable = true;
    listen_port.schedule = Some(task);
}

pub fn port_acceptable(listen_index: usize) -> bool {
    let listen_table = LISTEN_TABLE.borrow_mut();

    let listen_port = listen_table.get(listen_index);
    listen_port.map_or(false, |x| x.receivable)
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<()> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let listen_port = listen_table.values_mut().find(|p| p.port == port && p.receivable == true);
    if let Some(listen_port) = listen_port {
        let task = listen_port.schedule.clone().unwrap();
        // wakeup_task(MutRc::clone(&listen_port.schedule.clone().unwrap()));
        listen_port.schedule = None;
        listen_port.receivable = false;

        accept_connection(port, tcp_packet, task);
        Some(())
    } else {
        None
    }
}

pub fn accept_connection(_port: u16, tcp_packet: &TCPPacket, mut task: MutRc<Task>) {
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

// store in the fd_table, delete the listen table when close the application.
pub struct PortFd(usize);

impl PortFd {
    pub fn new(port_index: usize) -> Self {
        PortFd(port_index)
    }
}

impl Drop for PortFd {
    fn drop(&mut self) {
        LISTEN_TABLE.borrow_mut().remove(self.0);
    }
}

impl File for PortFd {
    fn read(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        0
    }

    fn write(&mut self, _buf: Vec<&'static mut [u8]>) -> usize {
        0
    }
}
