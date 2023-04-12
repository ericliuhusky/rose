use alloc::vec::Vec;
use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use mutrc::MutRc;

use crate::fs::File;
use crate::task::task::Task;

use super::tcp::TCP;

pub struct Port {
    pub port: u16,
    pub receivable: bool,
    pub schedule: Option<MutRc<Task>>,
}

lazy_static! {
    static ref LISTEN_TABLE: RefCell<Vec<Option<Port>>> = RefCell::new(Vec::new());
}

pub fn listen(port: u16) -> usize {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let mut index = usize::MAX;
    for i in 0..listen_table.len() {
        if listen_table[i].is_none() {
            index = i;
            break;
        }
    }

    let listen_port = Port {
        port,
        receivable: false,
        schedule: None,
    };

    if index == usize::MAX {
        listen_table.push(Some(listen_port));
        listen_table.len() - 1
    } else {
        listen_table[index] = Some(listen_port);
        index
    }
}

// can accept request
pub fn accept(listen_index: usize, task: MutRc<Task>) {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    assert!(listen_index < listen_table.len());
    let listen_port = listen_table[listen_index].as_mut();
    assert!(listen_port.is_some());
    let listen_port = listen_port.unwrap();
    listen_port.receivable = true;
    listen_port.schedule = Some(task);
}

pub fn port_acceptable(listen_index: usize) -> bool {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    assert!(listen_index < listen_table.len());

    let listen_port = listen_table[listen_index].as_mut();
    listen_port.map_or(false, |x| x.receivable)
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<()> {
    let mut listen_table = LISTEN_TABLE.borrow_mut();
    let mut listen_ports: Vec<&mut Option<Port>> = listen_table
        .iter_mut()
        .filter(|x| match x {
            Some(t) => t.port == port && t.receivable == true,
            None => false,
        })
        .collect();
    if listen_ports.len() == 0 {
        None
    } else {
        let listen_port = listen_ports[0].as_mut().unwrap();
        let task = listen_port.schedule.clone().unwrap();
        // wakeup_task(MutRc::clone(&listen_port.schedule.clone().unwrap()));
        listen_port.schedule = None;
        listen_port.receivable = false;

        accept_connection(port, tcp_packet, task);
        Some(())
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
        LISTEN_TABLE.borrow_mut()[self.0] = None
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
