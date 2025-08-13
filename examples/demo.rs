use std::time::Duration;

use cli_status_board::{SBState, SBStateConfig, Status, TaskNameWidth};

fn main() {
    let state = SBState::new(SBStateConfig {
        silent: false,
        task_name_width: TaskNameWidth::ExactRatio(0.25),
        ..Default::default()
    });

    // The handles for these tasks are immediately dropped, so we automatically remove them.
    // Except for error/info, which we display for a little while before removing automatically.
    {
        {
            let task_id = state.add_task("finished task", Status::Finished);
            state.add_subtask(&task_id, Status::Started);
        }
        let _ = state.add_task("error task", Status::Error);
        let _ = state.add_task("started task", Status::Started);
        let _ = state.add_task("queued task", Status::Queued);
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

    tasks.push(std::thread::spawn({
        let state = state.clone();
        move || {
            let task_id = state.add_task(
                format!("Task with looooooooooooooooooooooooooooooooooooooong message"),
                Status::Started,
            );
            std::thread::sleep(Duration::from_secs(10));
            state.update_task(&task_id, Status::Finished);
        }
    }));

    tasks.push(std::thread::spawn({
        let state = state.clone();
        move || {
            let task_id = state.add_task(format!("Task with no children"), Status::Started);
            std::thread::sleep(Duration::from_secs(10));
            state.update_task(&task_id, Status::Finished);
        }
    }));

    tasks.push(std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(4));
        state.info("Here's an informational message");
        std::thread::sleep(Duration::from_secs(8));
        state.info("Here's another one :)");
    }));

    for task in tasks {
        task.join().unwrap();
    }
}
