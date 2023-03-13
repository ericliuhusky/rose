use core::mem::MaybeUninit;

pub struct Once<T> {
    first_flag: bool,
    data: MaybeUninit<T>,
}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            first_flag: true,
            data: MaybeUninit::uninit(),
        }
    }

    pub fn once<F>(&mut self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.first_flag {
            self.first_flag = false;
            unsafe {
                *(&mut self.data as *mut MaybeUninit<T> as *mut T) = f();
            }
        }

        unsafe { &*(&self.data as *const MaybeUninit<T> as *const T) }
    }
}
