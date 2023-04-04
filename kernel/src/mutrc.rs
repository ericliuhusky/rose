use alloc::rc::{Rc, Weak};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct MutRc<T> {
    rc: Rc<UnsafeCell<T>>,
}

impl<T> MutRc<T> {
    pub fn new(value: T) -> Self {
        Self {
            rc: Rc::new(UnsafeCell::new(value)),
        }
    }

    pub fn downgrade(&self) -> MutWeak<T> {
        MutWeak {
            weak: Rc::downgrade(&self.rc),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone(),
        }
    }
}

impl<T> Deref for MutRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.rc.get()) }
    }
}

impl<T> DerefMut for MutRc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.rc.get()) }
    }
}

pub struct MutWeak<T> {
    weak: Weak<UnsafeCell<T>>,
}

impl<T> MutWeak<T> {
    pub fn upgrade(&self) -> Option<MutRc<T>> {
        match self.weak.upgrade() {
            Some(rc) => Some(MutRc { rc }),
            None => None,
        }
    }
}
