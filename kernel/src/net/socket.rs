use core::cell::RefCell;
use lazy_static::lazy_static;
use lose_net_stack::IPv4;

use crate::task::id::IDAllocDict;

// TODO: specify the protocol, TCP or UDP
pub struct Socket {
    pub raddr: IPv4,                // remote address
    pub lport: u16,                 // local port
    pub rport: u16,                 // rempote port
}

lazy_static! {
    static ref SOCKET_TABLE: RefCell<IDAllocDict<Socket>> = RefCell::new(IDAllocDict::new());
}

pub fn get_socket(raddr: IPv4, lport: u16, rport: u16) -> Option<usize> {
    let socket_table = SOCKET_TABLE.borrow_mut();
    socket_table
        .iter()
        .find(|(_, sock)| sock.raddr == raddr && sock.lport == lport && sock.rport == rport)
        .map_or(None, |(id, _)| Some(*id))
}

pub fn add_socket(raddr: IPv4, lport: u16, rport: u16) -> Option<usize> {
    if get_socket(raddr, lport, rport).is_some() {
        return None;
    }

    let mut socket_table = SOCKET_TABLE.borrow_mut();

    let socket = Socket {
        raddr,
        lport,
        rport,
    };

    Some(socket_table.insert(socket))
}

pub fn remove_socket(index: usize) {
    let mut socket_table = SOCKET_TABLE.borrow_mut();

    socket_table.remove(index);
}
