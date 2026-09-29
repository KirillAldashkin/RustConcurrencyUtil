use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct SpinLock<T = ()>(AtomicBool, UnsafeCell<T>);

// SAFETY: UnsafeCell is only accessed when locked atomically.
unsafe impl Sync for SpinLock {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        Self(AtomicBool::new(false), UnsafeCell::new(data))
    }

    pub fn get(&mut self) -> &mut T {
        self.1.get_mut()
    }

    pub fn lock<'a>(&'a self) -> SpinLockScope<'a, T> {
        while self.0.swap(true, Ordering::AcqRel) {}
        // SAFETY: spinlock above guarantees unique access
        let locked = unsafe { self.1.get().as_mut_unchecked() };
        SpinLockScope(&self.0, locked)
    }
}

#[derive(Debug)]
pub struct SpinLockScope<'a, T>(&'a AtomicBool, &'a mut T);

impl<'a, T> Drop for SpinLockScope<'a, T> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinLockScope<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.1
    }
}

impl<'a, T> DerefMut for SpinLockScope<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1
    }
}
