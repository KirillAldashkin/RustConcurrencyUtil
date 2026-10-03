#[cfg(feature = "alloc")]
mod onceevent;
#[cfg(feature = "alloc")]
pub use onceevent::OnceEvent;

pub trait Event {
    type Wait: Future;

    fn fire(self);
    fn wait(self) -> Self::Wait;
}
