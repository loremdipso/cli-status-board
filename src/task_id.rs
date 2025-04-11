use std::{
    fmt::Display,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
        mpsc::Sender,
    },
};

use crate::{Status, state::TaskEvent};

static LATEST_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Clone)]
pub struct TaskId {
    id: i32,
    maybe_sender: Option<Arc<Sender<TaskEvent>>>,
}

impl TaskId {
    pub(crate) fn new() -> Self {
        let id = LATEST_ID.fetch_add(1, Ordering::SeqCst) + 1;
        Self {
            id,
            maybe_sender: None,
        }
    }

    pub(crate) fn new_with_sender(sender: Sender<TaskEvent>) -> Self {
        let id = LATEST_ID.fetch_add(1, Ordering::SeqCst) + 1;
        Self {
            id,
            maybe_sender: Some(Arc::new(sender)),
        }
    }

    pub(crate) fn make_weak(&self) -> TaskId {
        Self {
            id: self.id,
            maybe_sender: None,
        }
    }
}

impl Eq for TaskId {}
impl PartialEq for TaskId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Drop for TaskId {
    fn drop(&mut self) {
        if let Some(sender_rc) = self.maybe_sender.take() {
            // This is about to drop, so let's go ahead and mark this task as "finished".
            if let Some(sender) = Arc::into_inner(sender_rc) {
                sender
                    // Don't pass the sender in order to avoid infinite loops
                    .send(TaskEvent::UpdateTask(self.make_weak(), Status::Finished))
                    .unwrap();
            }
        }
    }
}
