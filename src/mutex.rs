use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use sync::Semaphore;
use error::KernelResult;

pub struct Mutex<T: Send> {
    sem: Semaphore,
    data: UnsafeCell<T>
}

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T: Send> Mutex<T> {
    pub fn new(data: T) -> KernelResult<Mutex<T>> {
        let sem = Semaphore::new()?;
        sem.up();

        Ok(Mutex {
            sem: sem,
            data: UnsafeCell::new(data)
        })
    }

    pub fn lock<'a>(&'a self) -> KernelResult<MutexGuard<'a, T>> {
        self.sem.down()?;

        Ok(MutexGuard {
            sem: &self.sem,
            data: unsafe { &mut *self.data.get() }
        })
    }
}

pub struct MutexGuard<'a, T: 'a> {
    sem: &'a Semaphore,
    data: &'a mut T
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.sem.up();
    }
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}
