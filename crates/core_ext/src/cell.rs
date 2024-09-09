use core::cell::UnsafeCell;

/// # 提供最简单的内部可变性，实现`Sync`以支持`static`变量  
/// `UnsafeCell`不支持`static`变量，仅支持`static mut`
pub struct SafeCell<T> {
    cell: UnsafeCell<T>,
}

unsafe impl<T> Sync for SafeCell<T> {}

impl<T> SafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            cell: UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> *mut T {
        self.cell.get()
    }

    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.get() }
    }
}
