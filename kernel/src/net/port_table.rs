use alloc::collections::BTreeSet;
use core::cell::RefCell;

static_var! {
    LISTEN_PORTS: RefCell<BTreeSet<u16>> = RefCell::new(BTreeSet::new());
}

pub fn listen(port: u16) {
    let mut listen_table = LISTEN_PORTS.borrow_mut();
    listen_table.insert(port);
}
