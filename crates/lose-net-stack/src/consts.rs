// ip packet
pub(crate) const IP_PROTOCAL_TCP: u8 = 6;
pub(crate) const IP_PROTOCAL_UDP: u8 = 17;

pub(crate) const IP_HEADER_VHL: u8 = 4 << 4 | 20 >> 2;

pub(crate) const TCP_EMPTY_DATA: &[u8] = &[];