use alloc::collections::BTreeSet;
use core_ext::cell::SafeCell;

static_var! {
    LISTEN_PORTS: SafeCell<BTreeSet<u16>> = SafeCell::new(BTreeSet::new());
}

pub fn listen(port: u16) {
    let listen_table = LISTEN_PORTS.borrow_mut();
    listen_table.insert(port);
}
