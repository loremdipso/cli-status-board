use std::time::Duration;

use cli_status_board::{State, Status, start_state_display_thread};

fn main() {
    let state = State::new();
    start_state_display_thread(state.clone());

    let tasks = (0..10)
        .into_iter()
        .map(|index| {
            let state = state.clone();
            std::thread::spawn(move || {
                let key = format!("Key_{index}");
                std::thread::sleep(Duration::from_secs(index));
                if let Ok(mut state) = state.lock() {
                    state.add_task(&key, Status::Queued);
                }
                std::thread::sleep(Duration::from_secs(index));
                if let Ok(mut state) = state.lock() {
                    for i in 0..10 {
                        state.add_subtask(&key, format!("subkey_key_{i}"), Status::Started);
                    }
                }
                for i in 0..10 {
                    std::thread::sleep(Duration::from_secs(index));
                    if let Ok(mut state) = state.lock() {
                        state.update_subtask(&key, format!("subkey_key_{i}"), Status::Finished);
                    }
                }

                if let Ok(mut state) = state.lock() {
                    state.update_task(&key, Status::Finished);
                }
            })
        })
        .collect::<Vec<_>>();

    for task in tasks {
        task.join().unwrap();
    }
}
