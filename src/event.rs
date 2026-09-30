#[cfg(feature = "alloc")]
mod onceevent;
#[cfg(feature = "alloc")]
pub use onceevent::OnceEvent;

pub trait Event: Future {
    fn fire(self);
}
