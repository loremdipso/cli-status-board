use crate::{Status, TaskId, internal_state::InternalState};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Task {
    pub key: TaskId,
    pub display_name: Option<String>,
    pub time: std::time::Instant,
    pub substate: InternalState,
}

impl Task {
    pub fn substate_status(&self) -> String {
        let num_finished = match self.substate.task_map.get(&Status::Finished) {
            Some(v) => v.len(),
            None => 0,
        };

        let total = self.substate.get_total();
        if total == 0 {
            return "".into();
        }
        return format!("({} / {})", num_finished, total);
    }
}
