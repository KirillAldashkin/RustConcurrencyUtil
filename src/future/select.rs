use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::Either;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Select<A: Future, B: Future>(pub A, pub B);

impl<A: Future, B: Future> Future for Select<A, B> {
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: doing projections, fields of pinned are pinned
        let (a, b) = unsafe {
            let ptr = self.get_unchecked_mut() as *mut Self;
            (
                Pin::new_unchecked(&mut (*ptr).0),
                Pin::new_unchecked(&mut (*ptr).1),
            )
        };

        if let Poll::Ready(a) = a.poll(cx) {
            Poll::Ready(Either::A(a))
        } else if let Poll::Ready(b) = b.poll(cx) {
            Poll::Ready(Either::B(b))
        } else {
            Poll::Pending
        }
    }
}
