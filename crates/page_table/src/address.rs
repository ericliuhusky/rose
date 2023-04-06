pub type VPN = VirtualPage;
pub type PPN = PhysicalPage;
pub type VA = VirtualAddress;
pub type PA = PhysicalAddress;

pub trait Page {
    type Address: Address;

    fn new(n: usize) -> Self;

    fn number(&self) -> usize;

    fn start_addr(&self) -> Self::Address {
        Self::Address::new(self.number() << 12)
    }

    fn end_addr(&self) -> Self::Address {
        Self::Address::new((self.number() + 1) << 12)
    }
}

#[derive(Clone, Copy)]
pub struct VirtualPage(usize);

impl Page for VirtualPage {
    type Address = VirtualAddress;

    fn new(n: usize) -> Self {
        Self(n)
    }

    fn number(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct PhysicalPage(usize);

impl Page for PhysicalPage {
    type Address = PhysicalAddress;

    fn new(n: usize) -> Self {
        Self(n)
    }

    fn number(&self) -> usize {
        self.0
    }
}

impl VirtualPage {
    pub fn indexs(&self) -> [usize; 3] {
        [
            (self.0 >> 18) & 0x1ff,
            (self.0 >> 9) & 0x1ff,
            self.0 & 0x1ff,
        ]
    }
}

pub trait Address {
    type Page: Page;

    fn new(n: usize) -> Self;

    fn number(&self) -> usize;

    fn page(&self) -> Self::Page {
        Self::Page::new(self.number() >> 12)
    }

    fn align_to_lower(&self) -> Self
    where
        Self: Sized,
    {
        Self::new(self.number() & !0xfff)
    }

    fn align_to_upper(&self) -> Self
    where
        Self: Sized,
    {
        Self::new((self.number() + 0xfff) & !0xfff)
    }

    fn page_offset(&self) -> usize {
        self.number() & 0xfff
    }

    fn offset(&self, page_offset: usize) -> Self
    where
        Self: Sized,
    {
        Self::new(self.number() + page_offset)
    }
}

#[derive(Clone, Copy)]
pub struct VirtualAddress(usize);

impl Address for VirtualAddress {
    type Page = VirtualPage;

    fn new(n: usize) -> Self {
        Self(n)
    }

    fn number(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct PhysicalAddress(usize);

impl Address for PhysicalAddress {
    type Page = PhysicalPage;

    fn new(n: usize) -> Self {
        Self(n)
    }

    fn number(&self) -> usize {
        self.0
    }
}
