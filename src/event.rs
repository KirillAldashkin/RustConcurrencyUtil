#[cfg(feature = "alloc")]
mod onceevent;
#[cfg(feature = "alloc")]
pub use onceevent::OnceEvent;

pub trait Event {
    type Wait<'a>: Future where Self: 'a;

    fn fire(&self);
    fn wait<'a>(&'a self) -> Self::Wait<'a>;
}
