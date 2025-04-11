use std::time::Duration;

use cli_status_board::{SBState, SBStateConfig, Status};

fn main() {
    let state = SBState::new(SBStateConfig {
        silent: false,
        ..Default::default()
    });

    // When this handle drops the task completes
    let _handle = state.add_task(format!("Some super basic task"), Status::Started);

    std::thread::sleep(Duration::from_secs(10));
}
