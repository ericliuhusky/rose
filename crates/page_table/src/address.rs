use core::marker::PhantomData;

pub trait Space: Clone + Copy {}

#[derive(Clone, Copy)]
pub struct PageNumber<S>(pub usize, PhantomData<S>);

#[derive(Clone, Copy)]
pub struct Address<S>(pub usize, PhantomData<S>);

#[derive(Clone, Copy)]
pub struct Virtual;

#[derive(Clone, Copy)]
pub struct Physical;

impl Space for Virtual {}
impl Space for Physical {}

pub type VPN = PageNumber<Virtual>;
pub type PPN = PageNumber<Physical>;
pub type VA = Address<Virtual>;
pub type PA = Address<Physical>;

impl<S: Space> PageNumber<S> {
    pub fn new(n: usize) -> Self {
        Self(n, PhantomData)
    }

    pub fn start_addr(&self) -> Address<S> {
        Address::new(self.0 << 12)
    }

    pub fn end_addr(&self) -> Address<S> {
        Address::new((self.0 + 1) << 12)
    }
}

impl VPN {
    pub fn indexs(&self) -> [usize; 3] {
        [
            (self.0 >> 18) & 0x1ff,
            (self.0 >> 9) & 0x1ff,
            self.0 & 0x1ff
        ]
    }
}

impl<S: Space> Address<S> {
    pub fn new(n: usize) -> Self {
        Self(n, PhantomData)
    }

    pub fn page_number(&self) -> PageNumber<S> {
        PageNumber::new(self.0 >> 12)
    }

    pub fn align_to_lower(&self) -> Self {
        Self::new(self.0 & !0xfff)
    }

    pub fn align_to_upper(&self) -> Self {
        Self::new((self.0 + 0xfff) & !0xfff)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & 0xfff
    }

    pub fn offset(&self, page_offset: usize) -> Self {
        Self::new(self.0 + page_offset)
    }
}
