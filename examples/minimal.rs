use std::time::Duration;

use cli_status_board::{SBStateConfig, State, Status};

fn main() {
    let state = State::new(SBStateConfig {
        silent: false,
        ..Default::default()
    });

    // When this handle drops the task completes
    let _handle = state.add_task(format!("Some super basic task"), Status::Started);

    std::thread::sleep(Duration::from_secs(10));
}
