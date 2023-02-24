use std::sync::atomic::{AtomicUsize, Ordering};

static MESSAGEABLE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Messageable(usize);

pub fn new_id() -> Messageable {
    Messageable(MESSAGEABLE_ID.fetch_add(1, Ordering::Relaxed))
}
