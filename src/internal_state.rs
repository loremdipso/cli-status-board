use colored::ColoredString;
use rustc_hash::FxHashMap;

use crate::{Status, TaskId, task::Task};

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

    pub(crate) fn print_list<F>(&self, status: Status, max: usize, color_func: Option<F>)
    where
        F: Fn(&str) -> ColoredString,
    {
        if let Some(jobs) = self.task_map.get(&status) {
            if jobs.len() > 0 {
                println!("\n{:?} ({}):", status, jobs.len());

                for job in jobs.iter().take(max) {
                    let name = if let Some(display_name) = &job.display_name {
                        display_name.to_string()
                    } else {
                        job.key.to_string()
                    };

                    if let Some(color_func) = &color_func {
                        print!("\t{}", color_func(&name));
                    } else {
                        print!("\t{}", name);
                    }

                    let status = job.substate_status();
                    if status.len() > 0 {
                        print!(" {}", status);
                    }
                    println!();
                }
                if jobs.len() > max {
                    println!("\t...");
                }
            }
        }
    }
}
