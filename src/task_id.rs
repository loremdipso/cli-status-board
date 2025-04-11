use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicI32, Ordering},
        mpsc::Sender,
    },
};

use crate::state::TaskEvent;

static LATEST_ID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug)]
pub struct TaskId {
    id: i32,
    maybe_sender: Option<Sender<TaskEvent>>,
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
            maybe_sender: Some(sender),
        }
    }
}

impl Eq for TaskId {}
impl PartialEq for TaskId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Clone for TaskId {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            maybe_sender: None,
        }
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Drop for TaskId {
    fn drop(&mut self) {
        if let Some(sender) = &self.maybe_sender {
            sender
                .send(TaskEvent::DeleteTask(Self {
                    id: self.id,
                    maybe_sender: None,
                }))
                .unwrap();
        }
    }
}
