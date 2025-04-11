use crate::{Status, TaskId, internal_state::InternalState};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Task {
    pub key: TaskId,
    pub display_name: Option<String>,
    pub time: std::time::Instant,
    pub substate: InternalState,
}

impl Task {
    pub fn num_substate_total(&self) -> usize {
        self.substate.get_total()
    }

    pub fn num_substate_finished(&self) -> usize {
        match self.substate.task_map.get(&Status::Finished) {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
