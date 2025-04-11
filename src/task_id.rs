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
        if let Some(sender_rc) = &self.maybe_sender {
            // This is about to drop, so let's go ahead and delete this task.
            // I'm not exactly sure why 2 seems to be the right number here...
            if Arc::strong_count(sender_rc) <= 2 {
                sender_rc
                    // Don't pass the sender in order to avoid infinite loops
                    .send(TaskEvent::UpdateTask(
                        Self {
                            id: self.id,
                            maybe_sender: None,
                        },
                        Status::Finished,
                    ))
                    .unwrap();
            }
        }
    }
}
