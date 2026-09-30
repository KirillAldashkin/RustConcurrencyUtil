mod onceevent;
pub use onceevent::OnceEvent;

pub trait Event {
    fn fire(&self);
}
