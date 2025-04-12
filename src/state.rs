use crate::{Status, TaskId, column::ColumnFit, internal_state::InternalState};
use colored::Colorize;
use std::{sync::mpsc::Sender, time::Duration};

#[derive(Debug, Clone)]
pub struct SBState {
    sender: Sender<TaskEvent>,
}

// Configuration for the status board.
#[derive(Debug, Clone)]
pub struct SBStateConfig {
    // Defines how we should render the task name.
    // If unset then we'll restrict it to 50% of the available screen.
    pub task_name_width: TaskNameWidth,

    // Custom refresh rate for the status board. Defaults to 30 ms.
    // Since this only actually rerenders when the terminal size has
    // changed or there's a pending event this tends to be fine, but
    // you can turn this down for better performance.
    pub refresh_rate: Duration,

    // If true then the status board won't actually render anything.
    // Not terribly useful, apart from testing.
    pub silent: bool,
}

#[derive(Debug, Clone)]
pub enum TaskNameWidth {
    // Take up at least x% of the screen, growing if necessary
    Min(f32),

    // Take up at most x% of the screen, shrinking if possible
    Max(f32),

    // Take up exactly x% of the screen
    ExactRatio(f32),

    // Take up exactly x chars of the screen
    ExactChars(usize),
}

impl Default for SBStateConfig {
    fn default() -> Self {
        Self {
            silent: false,
            refresh_rate: Duration::from_millis(30),
            task_name_width: TaskNameWidth::Max(0.5),
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

impl SBState {
    pub fn new(config: SBStateConfig) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<TaskEvent>();

        std::thread::spawn(move || -> ! {
            let mut internal_state = InternalState::default();
            let mut should_refresh_display = true;
            let mut old_width = 0;
            let mut old_height = 0;
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

                if let Ok((width, height)) = termion::terminal_size() {
                    if width != old_width || height != old_height {
                        old_height = height;
                        old_width = width;
                        should_refresh_display = true;
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

                        let width = width as usize;
                        let task_name_fit = match config.task_name_width {
                            TaskNameWidth::Min(max) => {
                                ColumnFit::MIN((max.min(1.0) * width as f32) as usize)
                            }
                            TaskNameWidth::Max(max) => {
                                ColumnFit::MAX((max.min(1.0) * width as f32) as usize)
                            }
                            TaskNameWidth::ExactRatio(max) => {
                                ColumnFit::EXACT((max.min(1.0) * width as f32) as usize)
                            }
                            TaskNameWidth::ExactChars(max) => ColumnFit::EXACT(max.min(width)),
                        };

                        internal_state.print_list(
                            Status::Info,
                            10,
                            |f: &str| f.into(),
                            width,
                            ColumnFit::EXACT(width),
                        );
                        internal_state.print_list(
                            Status::Started,
                            10,
                            |f: &str| f.bright_green(),
                            width,
                            task_name_fit,
                        );
                        internal_state.print_list(
                            Status::Queued,
                            10,
                            |f: &str| f.bright_yellow(),
                            width,
                            task_name_fit,
                        );
                        internal_state.print_list(
                            Status::Error,
                            10,
                            |f: &str| f.bright_red(),
                            width,
                            task_name_fit,
                        );
                    }
                }

                std::thread::sleep(config.refresh_rate);
                should_refresh_display = false;
            }
        });

        Self { sender }
    }

    pub fn error<S: ToString>(&self, display_name: S) {
        let task_id = TaskId::new();
        self.sender
            .send(TaskEvent::AddTask(
                task_id.clone(),
                Some(display_name.to_string()),
                Status::Error,
            ))
            .unwrap();
    }

    pub fn info<S: ToString>(&self, display_name: S) {
        let task_id = TaskId::new();
        self.sender
            .send(TaskEvent::AddTask(
                task_id.clone(),
                Some(display_name.to_string()),
                Status::Info,
            ))
            .unwrap();
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
