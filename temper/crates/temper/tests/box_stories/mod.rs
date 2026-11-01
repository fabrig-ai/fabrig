//! Crash stories of temper/docs/design/draft/playground-reset.md, 4, 6.2–6.4 and 8.
mod reset;
mod set_up;

use crate::support::{
    self, boxed,
    ctl::{self, Call, Step},
};
use skein_io::kernel::Exit;
use skein_world::end_to_end::Binary;
use std::path::Path;

/// Start again on the retained directories; ctl polls the named store state
/// under the box's shared deadline before the rest of the story proceeds.
fn restart_until<T>(
    directory: &Path,
    flags: Vec<String>,
    client: &Path,
    state: &'static [u8],
    steps: Vec<Step>,
    check: impl FnOnce(&Binary) -> T,
) -> boxed::CtlOutcome<T> {
    let mut restart = vec![Step::Run(Call::sign_in()), Step::Until(Call::new(&["forge", "list", "--json"]), state)];
    restart.extend(steps);
    let seen = boxed::run_ctl(directory, boxed::echo_steps(flags), client, restart, boxed::BOX_DEADLINE, check);
    assert_success(&seen);
    seen
}

fn assert_success<T>(seen: &boxed::CtlOutcome<T>) {
    let stderr = String::from_utf8_lossy(&seen.stderr);
    assert_eq!(seen.exit, Some(Exit::Code(0)), "{stderr}");
    assert_eq!(seen.left, Some(Vec::new()), "no process outlived the box: {stderr}");
    assert!(seen.checked.is_some(), "every ctl step finished before stopping: {stderr}");
    for call in &seen.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "ctl: {}; box: {stderr}", String::from_utf8_lossy(&call.stderr));
    }
}

fn document(config: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON")
}

fn group(data: &Path, forge: u64) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(data.join(format!("box/groups/{forge}"))).expect("group file"))
        .expect("group JSON")
}

fn rows(call: &support::Ended) -> Vec<serde_json::Value> {
    ctl::json(call)["forges"].as_array().expect("forges rows").clone()
}
