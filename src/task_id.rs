use std::{
    fmt::Display,
    sync::atomic::{AtomicI32, Ordering},
};

static LATEST_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaskId {
    id: i32,
}

impl TaskId {
    pub(crate) fn new() -> Self {
        let id = LATEST_ID.fetch_add(1, Ordering::SeqCst) + 1;
        Self { id }
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Drop for TaskId {
    fn drop(&mut self) {
        // TODO
    }
}
