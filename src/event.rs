#[cfg(feature = "alloc")]
mod onceevent;
#[cfg(feature = "alloc")]
pub use onceevent::OnceEvent;

pub trait Event where for<'a> &'a Self: Future {
    fn fire(&self);
}
