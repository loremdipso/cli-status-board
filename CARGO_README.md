# CLI Status Board

Quickly visualize ongoing asynchronous tasks on the command line.

## Usage

```rust
use cli_status_board::{SBStateConfig, SBState, Status};

fn main() {
    let state = SBState::new(SBStateConfig {
        silent: false,
        ..Default::default()
    });

    // Add some arbitrary task with an initial status.
    let task_id = state.add_task(format!("Some super basic task"), Status::Queued);

    // Update the task's status.
    state.update_task(&task_id, Status::Started);

    // TaskId does reference counting so that when the last one drops
    // the task completes automatically. Or you can do so explicitly:
    state.update_task(&task_id, Status::Finished);
}
```

## Examples

See the examples directory. The demo example can be run with:

```sh
cargo run --example demo
```
