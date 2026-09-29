use core::sync::atomic::{AtomicBool, Ordering};

pub struct OnceFlag(AtomicBool);

impl OnceFlag {
    pub fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn fire(&self) -> bool {
        self.0.swap(true, Ordering::AcqRel)
    }

    pub fn fired(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
