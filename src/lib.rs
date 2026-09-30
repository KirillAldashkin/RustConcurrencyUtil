#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod event;
pub mod sync;
pub mod future;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
