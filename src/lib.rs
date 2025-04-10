use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use termion::{clear, cursor};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct State {
    task_map: HashMap<Status, Vec<Task>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Queued,
    Started,
    Finished,
    Error,
    Info,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Task {
    key: String,
    display_name: Option<String>,
    time: time::Instant,
    substate: State,
}

impl Task {
    fn substate_status(&self) -> String {
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

impl State {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn get_total(&self) -> usize {
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

    pub fn event(&mut self, event: String) {
        self.add_task(event, Status::Info);
    }

    pub fn error(&mut self, error: String) {
        self.add_task(error, Status::Error);
    }

    pub fn add_task(&mut self, key: String, status: Status) {
        self.task_map.entry(status).or_default().push(Task {
            key,
            display_name: None,
            time: time::Instant::now(),
            substate: State::default(),
        });
    }

    pub fn delete_task(&mut self, key: &String) {
        for (_, tasks) in self.task_map.iter_mut() {
            tasks.retain_mut(|task| -> bool {
                if &task.key == key {
                    return false;
                }
                return true;
            });
        }
    }

    pub fn set_display_name(&mut self, key: &str, display_name: &str) {
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

    pub fn update_task(&mut self, key: String, new_status: Status) {
        let mut overall_to_move = vec![];

        for (status, tasks) in self.task_map.iter_mut() {
            if *status == new_status {
                continue;
            }

            tasks.retain_mut(|task| -> bool {
                if task.key == key {
                    task.time = time::Instant::now();
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

    pub fn add_subtask(&mut self, key: &String, subkey: &String, status: Status) {
        for (_, tasks) in self.task_map.iter_mut() {
            for task in tasks {
                if task.key == *key {
                    task.substate.add_task(subkey.clone(), status);
                }
            }
        }
    }

    pub fn update_subtask(&mut self, key: &String, subkey: &String, new_status: Status) {
        for (_, tasks) in self.task_map.iter_mut() {
            for task in tasks {
                if task.key == *key {
                    task.substate.update_task(subkey.into(), new_status);
                }
            }
        }
    }

    fn clear_old_entries(&mut self, max_duration: time::Duration, statuses: &[Status]) {
        let now = time::Instant::now();
        for (_, rows) in self
            .task_map
            .iter_mut()
            .filter(|(status, _)| statuses.contains(status))
        {
            rows.retain_mut(|r| now.duration_since(r.time) < max_duration);
        }
    }

    fn print_list<F>(&self, status: Status, max: usize, color_func: Option<F>)
    where
        F: Fn(&str) -> ColoredString,
    {
        if let Some(jobs) = self.task_map.get(&status) {
            if jobs.len() > 0 {
                println!("\n{:?} ({}):", status, jobs.len());

                for job in jobs.iter().take(max) {
                    let name = if let Some(display_name) = &job.display_name {
                        format!("{} (aka {})", job.key, display_name)
                    } else {
                        job.key.clone()
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

pub fn start_state_display_thread(state: Arc<Mutex<State>>) {
    thread::spawn(move || -> ! {
        let sleep_time = time::Duration::from_millis(1_000);
        loop {
            // Reset the display
            print!("{}", clear::All);
            print!("{}", cursor::Goto(0, 1));
            println!("Status...\n");

            if let Ok(mut state) = state.lock() {
                state.clear_old_entries(
                    time::Duration::from_secs(10),
                    &[Status::Error, Status::Info][..],
                );

                let num_finished = match state.task_map.get(&Status::Finished) {
                    Some(v) => v.len(),
                    None => 0,
                };

                println!(
                    "{} / {}",
                    format!("{}", num_finished).bright_green(),
                    state.get_total(),
                );

                state.print_list(Status::Started, 10, Some(|f: &str| f.bright_green()));
                state.print_list(Status::Queued, 10, Some(|f: &str| f.bright_yellow()));
                state.print_list(Status::Error, 10, Some(|f: &str| f.bright_red()));
                state.print_list(Status::Info, 10, None::<fn(&str) -> ColoredString>);
            }
            thread::sleep(sleep_time);
        }
    });
}
