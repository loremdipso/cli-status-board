mod column;
mod internal_state;
mod state;
mod task;
mod task_id;

pub use state::State;
pub use task_id::TaskId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Queued,
    Started,
    Finished,
    Error,
    Info,
}
impl Status {
    fn is_finished(&self) -> bool {
        match self {
            Status::Queued => false,
            Status::Started => false,
            Status::Finished => true,
            Status::Info => true,
            Status::Error => true,
        }
    }
}
