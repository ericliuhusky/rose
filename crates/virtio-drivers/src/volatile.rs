use core::ptr;

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Volatile<T: Copy>(T);

impl<T: Copy> Volatile<T> {
    pub fn read(&self) -> T {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::read_volatile(&self.0) }
    }

    pub fn write(&mut self, value: T) {
        // UNSAFE: Safe, as we know that our internal value exists.
        unsafe { ptr::write_volatile(&mut self.0, value) };
    }
}
