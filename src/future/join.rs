use core::{
    mem::ManuallyDrop,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Join<A: Future, B: Future> {
    done: bool,
    a: Result<ManuallyDrop<A::Output>, A>,
    b: Result<ManuallyDrop<B::Output>, B>,
}

impl<A: Future, B: Future> Join<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self {
            done: false,
            a: Err(a),
            b: Err(b),
        }
    }
}

impl<A: Future, B: Future> Future for Join<A, B> {
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done {
            panic!("Join polled after returning value")
        }

        // SAFETY: pin projecting fields, which will never move
        let (mut a, mut b, mut done) = unsafe {
            let ptr = self.get_unchecked_mut() as *mut Self;
            (
                Pin::new_unchecked(&mut (*ptr).a),
                Pin::new_unchecked(&mut (*ptr).b),
                Pin::new_unchecked(&mut (*ptr).done),
            )
        };

        let a = match pin_result(a.as_mut()) {
            Ok(result) => Some(result),
            Err(future) => match future.poll(cx) {
                Poll::Ready(done) => {
                    a.set(Ok(ManuallyDrop::new(done)));
                    // SAFETY: just set it to Ok()
                    Some(unsafe { pin_result(a).unwrap_unchecked() })
                }
                Poll::Pending => None,
            },
        };
        let b = match pin_result(b.as_mut()) {
            Ok(result) => Some(result),
            Err(future) => match future.poll(cx) {
                Poll::Ready(done) => {
                    b.set(Ok(ManuallyDrop::new(done)));
                    // SAFETY: just set it to Ok()
                    Some(unsafe { pin_result(b).unwrap_unchecked() })
                }
                Poll::Pending => None,
            },
        };

        match (a, b) {
            (Some(a), Some(b)) => {
                done.set(true);
                // SAFETY: we move out of pin but `a` and `b`
                // are never exposed as pinned so it is fine
                Poll::Ready(unsafe {
                    (
                        ManuallyDrop::take(a.get_unchecked_mut()),
                        ManuallyDrop::take(b.get_unchecked_mut()),
                    )
                })
            }
            _ => Poll::Pending,
        }
    }
}

fn pin_result<T, E>(value: Pin<&mut Result<T, E>>) -> Result<Pin<&mut T>, Pin<&mut E>> {
    // SAFETY: pin projecting arms, which will never move
    unsafe {
        match value.get_unchecked_mut() {
            Ok(ok) => Ok(Pin::new_unchecked(ok)),
            Err(err) => Err(Pin::new_unchecked(err)),
        }
    }
}

impl<A: Future, B: Future> Drop for Join<A, B> {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        if let Ok(a) = &mut self.a {
            // SAFETY: this is only taken from by `poll` but it also sets `done`
            unsafe { ManuallyDrop::drop(a) };
        } else if let Ok(b) = &mut self.b {
            // SAFETY: this is only taken from by `poll` but it also sets `done`
            unsafe { ManuallyDrop::drop(b) };
        }
    }
}
