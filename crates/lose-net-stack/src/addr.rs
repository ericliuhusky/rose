#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct IPv4(u32);

impl IPv4 {
    pub const fn new(a1: u8, a2: u8, a3: u8, a4: u8) -> Self {
        IPv4((a1 as u32) << 24 | (a2 as u32) << 16 | (a3 as u32) << 8 | (a4 as u32))
    }

    pub fn from_u32(ip: u32) -> Self {
        IPv4(ip)
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Default)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(addr: [u8; 6]) -> Self {
        MacAddress(addr)
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        self.0
    }
}
