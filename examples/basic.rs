use std::time::Duration;

use cli_status_board::{State, Status};

fn main() {
    let state = State::new();

    {
        _ = state.add_task("finished task", Status::Finished);
        _ = state.add_task("error task", Status::Error);
        _ = state.add_task("started task", Status::Started);
        _ = state.add_task("queued task", Status::Queued);
    }

    let tasks = (0..10)
        .into_iter()
        .map(|index| {
            let state = state.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(index));
                let key = state.add_task(format!("Task {index}"), Status::Started);
                std::thread::sleep(Duration::from_secs(index));
                let sub_tasks = (0..10)
                    .into_iter()
                    .map(|_| state.add_subtask(&key, Status::Started))
                    .collect::<Vec<_>>();

                for sub_task_id in sub_tasks {
                    std::thread::sleep(Duration::from_secs(index + 1));
                    state.update_subtask(&key, &sub_task_id, Status::Finished);
                }

                state.update_task(&key, Status::Finished);
            })
        })
        .collect::<Vec<_>>();

    for task in tasks {
        task.join().unwrap();
    }
}
