use crate::{Status, TaskId, internal_state::InternalState};
use colored::Colorize;
use std::{sync::mpsc::Sender, time::Duration};

#[derive(Debug, Clone)]
pub struct State {
    sender: Sender<TaskEvent>,
}

// Configuration for the status board.
#[derive(Debug, Clone)]
pub struct SBStateConfig {
    // If true then the status board won't actually render anything.
    // Not terribly useful, apart from testing.
    pub silent: bool,

    // Custom refresh rate for the status board. Defaults to 1 second.
    pub refresh_rate: Duration,

    // If set then we'll only show the first n characters of a task's name.
    // If unset then we'll restrict it to 1/3 of the available screen.
    // If the task's name is too long we'll truncate with a '...'.
    pub max_task_name_width: Option<usize>,
}

impl Default for SBStateConfig {
    fn default() -> Self {
        Self {
            silent: false,
            refresh_rate: Duration::from_secs(1),
            max_task_name_width: None,
        }
    }
}

// Used internally to pipe commands over an mpsc channel.
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
    pub fn new(config: SBStateConfig) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<TaskEvent>();

        std::thread::spawn(move || -> ! {
            let mut internal_state = InternalState::default();
            let mut should_refresh_display = true;
            loop {
                for event in receiver.try_iter() {
                    should_refresh_display = true;
                    match event {
                        TaskEvent::AddTask(key, maybe_display_name, status) => {
                            internal_state.add_task(key.make_weak(), maybe_display_name, status);
                        }
                        TaskEvent::SetTaskDisplayName(key, display_name) => {
                            internal_state.set_display_name(key.make_weak(), display_name);
                        }
                        TaskEvent::UpdateTask(key, status) => {
                            internal_state.update_task(key.make_weak(), status);
                        }
                        TaskEvent::DeleteTask(key) => {
                            internal_state.delete_task(key.make_weak());
                        }
                        TaskEvent::AddSubTask(key, subkey, maybe_display_name, status) => {
                            internal_state.add_subtask(
                                key.make_weak(),
                                subkey,
                                maybe_display_name,
                                status,
                            );
                        }
                        TaskEvent::UpdateSubTask(key, subkey, new_status) => {
                            internal_state.update_subtask(key.make_weak(), subkey, new_status);
                        }
                    }
                }

                if !config.silent && should_refresh_display {
                    // Reset the display
                    print!("{}", termion::clear::All);
                    print!("{}", termion::cursor::Goto(0, 1));

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

                    if let Ok((width, _height)) = termion::terminal_size() {
                        let width = width as usize;
                        let max_task_name_width =
                            config.max_task_name_width.unwrap_or(width / 3).min(width);
                        internal_state.print_list(
                            Status::Started,
                            10,
                            |f: &str| f.bright_green(),
                            width,
                            max_task_name_width,
                        );
                        internal_state.print_list(
                            Status::Queued,
                            10,
                            |f: &str| f.bright_yellow(),
                            width,
                            max_task_name_width,
                        );
                        internal_state.print_list(
                            Status::Error,
                            10,
                            |f: &str| f.bright_red(),
                            width,
                            max_task_name_width,
                        );
                        internal_state.print_list(
                            Status::Info,
                            10,
                            |f: &str| f.into(),
                            width,
                            max_task_name_width,
                        );
                    }
                }

                std::thread::sleep(config.refresh_rate);
                should_refresh_display = false;
            }
        });

        Self { sender }
    }

    pub fn error<S: ToString>(&self, display_name: S) -> TaskId {
        self.add_task(display_name, Status::Error)
    }

    pub fn info<S: ToString>(&self, display_name: S) -> TaskId {
        self.add_task(display_name, Status::Info)
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
