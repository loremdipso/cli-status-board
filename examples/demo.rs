use std::time::Duration;

use cli_status_board::{State, Status};

fn main() {
    let state = State::new();

    // The handles for these tasks are immediately dropped, so we automatically remove them.
    // Except for error/info, which we display for a little while before removing automatically.
    {
        {
            let task_id = state.add_task("finished task", Status::Finished);
            state.add_subtask(&task_id, Status::Started);
        }
        _ = state.add_task("error task", Status::Error);
        _ = state.add_task("started task", Status::Started);
        _ = state.add_task("queued task", Status::Queued);
    }

    let mut tasks = (0..10)
        .into_iter()
        .map(|index| {
            let state = state.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(index));
                let task_id = state.add_task(format!("Task {index}"), Status::Started);
                std::thread::sleep(Duration::from_secs(index.min(3)));
                let sub_tasks = (0..10)
                    .into_iter()
                    .map(|_| state.add_subtask(&task_id, Status::Started))
                    .collect::<Vec<_>>();

                for sub_task_id in sub_tasks {
                    std::thread::sleep(Duration::from_secs((index + 1).min(3)));
                    state.update_subtask(&task_id, &sub_task_id, Status::Finished);
                }

                state.update_task(&task_id, Status::Finished);
            })
        })
        .collect::<Vec<_>>();

    tasks.push(std::thread::spawn(move || {
        let task_id = state.add_task(format!("Task with no children"), Status::Started);
        std::thread::sleep(Duration::from_secs(10));
        state.update_task(&task_id, Status::Finished);
    }));

    for task in tasks {
        task.join().unwrap();
    }
}
