use colored::Colorize;
use std::sync::mpsc::Sender;
use termion::{clear, cursor};

use crate::{Status, TaskId, internal_state::InternalState};

#[derive(Debug, Clone)]
pub struct State {
    sender: Sender<TaskEvent>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum TaskEvent {
    AddTask(TaskId, Option<String>, Status),
    SetTaskDisplayName(TaskId, String),
    UpdateTask(TaskId, Status),
    DeleteTask(TaskId),
    AddSubTask(TaskId, TaskId, Option<String>, Status),
    UpdateSubTask(TaskId, TaskId, Status),
}

impl State {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        std::thread::spawn(move || -> ! {
            let mut internal_state = InternalState::default();
            let sleep_time = std::time::Duration::from_millis(1_000);
            let mut should_refresh_display = true;
            loop {
                for event in receiver.try_iter() {
                    should_refresh_display = true;
                    match event {
                        TaskEvent::AddTask(key, maybe_display_name, status) => {
                            internal_state.add_task(key.clone(), maybe_display_name, status);
                        }
                        TaskEvent::SetTaskDisplayName(key, display_name) => {
                            internal_state.set_display_name(key, display_name);
                        }
                        TaskEvent::UpdateTask(key, status) => {
                            internal_state.update_task(key, status);
                        }
                        TaskEvent::DeleteTask(key) => {
                            internal_state.delete_task(key);
                        }
                        TaskEvent::AddSubTask(key, subkey, maybe_display_name, status) => {
                            internal_state.add_subtask(key, subkey, maybe_display_name, status);
                        }
                        TaskEvent::UpdateSubTask(key, subkey, new_status) => {
                            internal_state.update_subtask(key, subkey, new_status);
                        }
                    }
                }

                if should_refresh_display {
                    // Reset the display
                    print!("{}", clear::All);
                    print!("{}", cursor::Goto(0, 1));

                    internal_state.clear_old_entries(
                        std::time::Duration::from_secs(10),
                        &[Status::Error, Status::Info],
                    );

                    let num_finished = match internal_state.task_map.get(&Status::Finished) {
                        Some(v) => v.len(),
                        None => 0,
                    };

                    println!(
                        "Finished tasks: {} / {}",
                        format!("{}", num_finished).bright_green(),
                        internal_state.get_total(),
                    );

                    internal_state.print_list(Status::Started, 10, |f: &str| f.bright_green());
                    internal_state.print_list(Status::Queued, 10, |f: &str| f.bright_yellow());
                    internal_state.print_list(Status::Error, 10, |f: &str| f.bright_red());
                    internal_state.print_list(Status::Info, 10, |f: &str| f.into());
                }

                std::thread::sleep(sleep_time);
            }
        });

        Self { sender }
    }

    pub fn add_task<S: ToString>(&self, display_name: S, status: Status) -> TaskId {
        let task_id = TaskId::new_with_sender(self.sender.clone());
        self.sender
            .send(TaskEvent::AddTask(
                task_id.clone(),
                Some(display_name.to_string()),
                status,
            ))
            .unwrap();
        return task_id;
    }

    pub fn set_task_display_name(&self, task_id: &TaskId, display_name: String) {
        self.sender
            .send(TaskEvent::SetTaskDisplayName(task_id.clone(), display_name))
            .unwrap();
    }

    pub fn delete_task(&self, task_id: &TaskId) {
        self.sender
            .send(TaskEvent::DeleteTask(task_id.clone()))
            .unwrap();
    }

    pub fn update_task(&self, task_id: &TaskId, new_status: Status) {
        self.sender
            .send(TaskEvent::UpdateTask(task_id.clone(), new_status))
            .unwrap();
    }

    pub fn add_subtask(&self, task_id: &TaskId, status: Status) -> TaskId {
        let sub_task_id = TaskId::new();
        self.sender
            .send(TaskEvent::AddSubTask(
                task_id.clone(),
                sub_task_id.clone(),
                None,
                status,
            ))
            .unwrap();
        return sub_task_id;
    }

    pub fn update_subtask(&self, task_id: &TaskId, sub_task_id: &TaskId, status: Status) {
        self.sender
            .send(TaskEvent::UpdateSubTask(
                task_id.clone(),
                sub_task_id.clone(),
                status,
            ))
            .unwrap();
    }
}
