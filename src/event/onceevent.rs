use core::{pin::Pin, task::{self, Waker}};

use alloc::{sync::Arc, vec::Vec};

use crate::sync::{OnceFlag, SpinLock};

#[derive(Debug)]
struct OnceInner {
    used: OnceFlag,
    wakers: SpinLock<Vec<Waker>>,
}

#[derive(Clone, Debug)]
pub struct OnceEvent(Arc<OnceInner>);

impl OnceEvent {
    pub fn new() -> Self {
        Self(Arc::new(OnceInner {
            used: OnceFlag::new(),
            wakers: SpinLock::new(Vec::new()),
        }))
    }

    pub fn fire(&self) {
        let mut wakers = self.0.wakers.lock();

        if self.0.used.fire() {
            return;
        }

        let wakers = core::mem::replace(&mut *wakers, Vec::new());
        for waker in wakers {
            waker.wake();
        }
    }
}

impl Future for OnceEvent {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut wakers = self.0.wakers.lock();

        if self.0.used.fired() {
            return task::Poll::Ready(());
        }

        wakers.push(cx.waker().clone());
        return task::Poll::Pending;
    }
}
