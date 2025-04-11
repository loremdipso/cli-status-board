use colored::ColoredString;
use rustc_hash::FxHashMap;

use crate::{
    Status, TaskId,
    column::{Column, ColumnAlign, ColumnConfig, ColumnFit},
    task::Task,
};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct InternalState {
    pub task_map: FxHashMap<Status, Vec<Task>>,
}

impl InternalState {
    pub(crate) fn get_total(&self) -> usize {
        let num_started = match self.task_map.get(&Status::Started) {
            Some(v) => v.len(),
            None => 0,
        };
        let num_queued = match self.task_map.get(&Status::Queued) {
            Some(v) => v.len(),
            None => 0,
        };
        let num_finished = match self.task_map.get(&Status::Finished) {
            Some(v) => v.len(),
            None => 0,
        };
        return num_finished + num_queued + num_started;
    }

    // pub(crate) fn event(&mut self, key: TaskId) {
    //     self.add_task(key, None, Status::Info);
    // }

    // pub(crate) fn error(&mut self, key: TaskId) {
    //     self.add_task(key, None, Status::Error);
    // }

    pub(crate) fn add_task(&mut self, key: TaskId, display_name: Option<String>, status: Status) {
        self.task_map.entry(status).or_default().push(Task {
            key,
            display_name,
            time: std::time::Instant::now(),
            substate: InternalState::default(),
        });
    }

    pub(crate) fn delete_task(&mut self, key: TaskId) {
        for (status, tasks) in self.task_map.iter_mut() {
            // Only delete tasks that aren't finished yet
            if !status.is_finished() {
                tasks.retain(|task| if task.key == key { false } else { true });
            }
        }
    }

    pub(crate) fn set_display_name(&mut self, key: TaskId, display_name: String) {
        for (_, tasks) in self.task_map.iter_mut() {
            for task in tasks {
                if task.key == key {
                    task.display_name = Some(display_name.to_string());
                }
            }
        }
    }

    // pub fn clear_display_name(&mut self, key: &str) {
    //     for (_, tasks) in self.task_map.iter_mut() {
    //         for task in tasks {
    //             if task.key == key {
    //                 task.display_name = None;
    //             }
    //         }
    //     }
    // }

    pub(crate) fn update_task(&mut self, key: TaskId, new_status: Status) {
        let mut overall_to_move = vec![];
        for (status, tasks) in self.task_map.iter_mut() {
            if *status == new_status {
                continue;
            }

            tasks.retain_mut(|task| -> bool {
                if task.key == key {
                    task.time = std::time::Instant::now();
                    overall_to_move.push(task.clone());
                    return false;
                }
                return true;
            });
        }

        self.task_map
            .entry(new_status)
            .or_default()
            .extend(overall_to_move);
    }

    pub(crate) fn add_subtask(
        &mut self,
        key: TaskId,
        subkey: TaskId,
        display_name: Option<String>,
        status: Status,
    ) {
        for (_, tasks) in self.task_map.iter_mut() {
            for task in tasks {
                if task.key == key {
                    task.substate.add_task(subkey, display_name, status);
                    return;
                }
            }
        }
    }

    pub(crate) fn update_subtask(&mut self, key: TaskId, subkey: TaskId, new_status: Status) {
        for (_, tasks) in self.task_map.iter_mut() {
            for task in tasks {
                if task.key == key {
                    task.substate.update_task(subkey.clone(), new_status);
                }
            }
        }
    }

    pub(crate) fn clear_old_entries(
        &mut self,
        max_duration: std::time::Duration,
        statuses: &[Status],
    ) {
        let now = std::time::Instant::now();
        for (_, rows) in self
            .task_map
            .iter_mut()
            .filter(|(status, _)| statuses.contains(status))
        {
            rows.retain_mut(|r| now.duration_since(r.time) < max_duration);
        }
    }

    pub(crate) fn print_simple<F>(
        &self,
        status: Status,
        max: usize,
        color_func: F,
        terminal_width: usize,
    ) where
        F: Fn(&str) -> ColoredString,
    {
        if let Some(jobs) = self.task_map.get(&status) {
            if jobs.len() > 0 {
                println!("\n{:?} ({}):", status, jobs.len());

                let mut data = Column::new(ColumnConfig {
                    align: ColumnAlign::LEFT,
                    fit: ColumnFit::MAX(terminal_width),
                    left_padding: 4,
                    right_padding: 1,
                });
                for job in jobs.iter().take(max) {
                    let name = job
                        .display_name
                        .clone()
                        .unwrap_or_else(|| job.key.to_string());

                    data.push(color_func(&name));
                }

                for row in 0..data.len() {
                    println!("{}", data.to_string(row));
                }
            }
        }
    }

    pub(crate) fn print_complex<F>(
        &self,
        status: Status,
        max: usize,
        color_func: F,
        terminal_width: usize,
        max_task_name_width: usize,
    ) where
        F: Fn(&str) -> ColoredString,
    {
        if let Some(jobs) = self.task_map.get(&status) {
            if jobs.len() > 0 {
                println!("\n{:?} ({}):", status, jobs.len());

                let mut columns = vec![
                    // name
                    Column::new(ColumnConfig {
                        align: ColumnAlign::LEFT,
                        fit: ColumnFit::MAX(max_task_name_width),
                        left_padding: 4,
                        right_padding: 1,
                    }),
                    // # subjob finished
                    Column::new(ColumnConfig {
                        align: ColumnAlign::RIGHT,
                        fit: ColumnFit::NORMAL,
                        left_padding: 3,
                        right_padding: 1,
                    }),
                    // # subjob total
                    Column::new(ColumnConfig {
                        align: ColumnAlign::RIGHT,
                        fit: ColumnFit::NORMAL,
                        left_padding: 0,
                        right_padding: 1,
                    }),
                ];
                let mut progresses = Vec::new();

                let mut num_rows = 0;
                for job in jobs.iter().take(max) {
                    let name = job
                        .display_name
                        .clone()
                        .unwrap_or_else(|| job.key.to_string());

                    columns[0].push(color_func(&name));
                    if job.num_substate_total() == 0 {
                        columns[1].push("".into());
                        columns[2].push("".into());
                        progresses.push(None);
                    } else {
                        let total = job.num_substate_total();
                        let finished = job.num_substate_finished();
                        columns[1].push(format!("{} /", finished).into());
                        columns[2].push(total.to_string().into());
                        progresses.push(Some(finished as f32 / total as f32));
                    }
                    num_rows += 1;
                }

                if jobs.len() > max {
                    num_rows += 1;
                    columns[0].push("...".into());
                }

                for row_index in 0..num_rows {
                    let mut line = String::new();
                    let mut line_len = 0;
                    for column_index in 0..columns.len() {
                        line += &format!("{}", columns[column_index].to_string(row_index));
                        line_len += columns[column_index].line_len();
                    }
                    if row_index < progresses.len() {
                        if let Some(progress) = progresses[row_index] {
                            line += &get_progress_bar(
                                progress,
                                terminal_width.checked_sub(line_len).unwrap_or_default(),
                            );
                        }
                    }
                    println!("{line}");
                }
            }
        }
    }
}

fn get_progress_bar(progress: f32, available_width: usize) -> String {
    let width = (progress * available_width as f32).round() as usize;
    format!("{:=>width$}", ">")
}
