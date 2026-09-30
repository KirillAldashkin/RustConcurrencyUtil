use core::{
    mem::ManuallyDrop,
    pin::Pin,
    task::{self, Waker},
};

use alloc::vec::Vec;

use crate::{event::Event, sync::{OnceFlag, SpinLock}};

#[derive(Debug)]
pub struct OnceEvent {
    used: OnceFlag,
    wakers: SpinLock<ManuallyDrop<Vec<Waker>>>,
}

impl OnceEvent {
    pub fn new() -> Self {
        Self {
            used: OnceFlag::new(),
            wakers: SpinLock::new(ManuallyDrop::new(Vec::new())),
        }
    }
}

impl Event for OnceEvent {
    fn fire(&self) {
        if self.used.fire() {
            return;
        }

        let mut lock = self.wakers.lock();
        // SAFETY:
        // 1) Value is taken once - `used` is checked at the beginning
        // 2) Value is never used again - `poll` checks the same `used`
        let wakers = unsafe { ManuallyDrop::take(&mut lock) };
        drop(lock);

        for waker in wakers {
            waker.wake();
        }
    }
}

impl Drop for OnceEvent {
    fn drop(&mut self) {
        if !self.used.fired_mut() {
            // SAFETY: this is only dropped here and taken in `fire()`,
            // but it also sets the `used` flag which is checked above.
            unsafe { ManuallyDrop::drop(self.wakers.get()) };
        }
    }
}

impl Future for &OnceEvent {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        if self.used.fired() {
            return task::Poll::Ready(());
        }

        let mut wakers = self.wakers.lock();

        if self.used.fired() {
            return task::Poll::Ready(());
        }

        wakers.push(cx.waker().clone());
        return task::Poll::Pending;
    }
}
