#![cfg(feature = "alloc")]

use core::{
    mem::ManuallyDrop,
    pin::Pin,
    task::{self, Waker},
};

use alloc::{sync::Arc, vec::Vec};

use crate::sync::{OnceFlag, SpinLock};

#[derive(Debug)]
struct OnceInner {
    used: OnceFlag,
    wakers: SpinLock<ManuallyDrop<Vec<Waker>>>,
}

impl Drop for OnceInner {
    fn drop(&mut self) {
        if !self.used.fired_mut() {
            // SAFETY: this is only dropped here and taken in `fire()`,
            // but it also sets the `used` flag which is checked above.
            unsafe { ManuallyDrop::drop(self.wakers.get()) };
        }
    }
}

#[derive(Clone, Debug)]
pub struct OnceEvent(Arc<OnceInner>);

impl OnceEvent {
    pub fn new() -> Self {
        Self(Arc::new(OnceInner {
            used: OnceFlag::new(),
            wakers: SpinLock::new(ManuallyDrop::new(Vec::new())),
        }))
    }

    pub fn fire(&self) {
        if self.0.used.fire() {
            return;
        }

        let mut lock = self.0.wakers.lock();
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

impl Future for OnceEvent {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        if self.0.used.fired() {
            return task::Poll::Ready(());
        }

        let mut wakers = self.0.wakers.lock();

        if self.0.used.fired() {
            return task::Poll::Ready(());
        }

        wakers.push(cx.waker().clone());
        return task::Poll::Pending;
    }
}
