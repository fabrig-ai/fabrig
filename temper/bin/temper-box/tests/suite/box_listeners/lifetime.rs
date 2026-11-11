//! Shipped boxes inherit ignored hangups (temper/docs/design/protocol/setup/testing.md, 1).
use super::box_link;
use crate::support::{boxed, parent};

#[test]
fn a_box_ignoring_hangups_runs_on() {
    let mut command = std::process::Command::new("sh");
    command.args(["-c", r#"trap "" HUP; exec "$0" "$@""#]);
    command.arg(skein_scratch::paths::binary("temper-box"));
    let mut boxed = box_link::Boxed::with_command("ignored-hangup", command);
    let link = boxed.link();
    boxed.wait(|boxed| boxed::serve_listening(&boxed.root().join("data")));
    parent::signal(boxed.guard.child.as_mut().expect("box child"), "HUP");
    assert!(box_link::signs_in(boxed.root(), "after-hangup", &link), "the ignored-hangup box still answers");
    assert!(boxed.guard.child.as_mut().expect("box child").try_wait().expect("box status").is_none());
    let (_, _, log) = boxed.stop();
    assert!(log.lines().last().expect("last log line").ends_with("Stopped"), "SIGTERM still stops the box");
}
