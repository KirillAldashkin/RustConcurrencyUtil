#[cfg(feature = "alloc")]
mod onceevent;
#[cfg(feature = "alloc")]
pub use onceevent::OnceEvent;

pub trait Event {
    fn fire(self);
    fn wait(self) -> impl Future;
}
