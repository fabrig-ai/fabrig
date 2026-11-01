//! Whole resets across box cuts (temper/docs/design/draft/playground-reset.md, 4, 6.2, 8).
use super::playground;
use crate::support::{
    self, Ended,
    ctl::{self, Call, Step},
};
use skein_io::kernel::Exit;
use std::path::Path;

#[test]
fn the_box_killed_during_a_reset_ends_it_whole_at_its_next_start() {
    reset_cut(false);
}

#[test]
fn the_box_killed_twice_during_a_reset_ends_it_whole() {
    reset_cut(true);
}

fn reset_cut(twice: bool) {
    let scratch = support::Scratch::new(if twice { "reset-cut-twice" } else { "reset-cut-once" });
    let (paths, flags) = playground::initialize(&scratch);
    let mut steps = playground::running_steps(&paths[1]);
    steps.extend([
        Step::Start(Call::new(&["playground", "reset", "--key", "cut-reset"])),
        Step::KillBox,
        Step::StartBox,
    ]);
    if twice {
        steps.extend([
            Step::Start(Call::new(&["playground", "reset", "--key", "second-cut-reset"])),
            Step::KillBox,
            Step::StartBox,
        ]);
    }
    steps.extend([
        Step::Until(read_not_started(&scratch.path().join("client")), b"reset-read exit 2\n"),
        Step::Run(Call::new(&["forge", "list"]).json()),
        Step::Run(Call::new(&["account", "list"]).json()),
        Step::Run(Call::new(&["me"]).json()),
    ]);
    let seen =
        support::boxed::run_ctl(scratch.path(), flags, scratch.path(), steps, support::boxed::BOX_DEADLINE, |_| ());
    let stderr = String::from_utf8_lossy(&seen.stderr);
    assert_eq!(seen.checked, Some(()), "all calls completed: {stderr}");
    assert_eq!(seen.exit, Some(Exit::Code(0)), "box stopped cleanly: {stderr}");
    assert_eq!(seen.left, Some(Vec::new()), "nothing outlived the box: {stderr}");
    assert_eq!(seen.runs.len(), if twice { 3 } else { 2 }, "each restart retained evidence");
    for run in &seen.runs[..seen.runs.len() - 1] {
        assert_eq!(run.exit, Some(Exit::Signal(9)), "whole box killed");
    }
    let calls = &seen.calls;
    let forge = playground::running_forge(&calls[2]);
    playground::assert_running_accounts(&calls[3]);
    playground::assert_playground_project(&calls[4], true);
    assert_ne!(calls[5].stdout, b"reset\n", "immediate cut missed the reset");
    eprintln!("first kill: immediate-cut fallback after starting keyed reset; reset phase and resumption not shown");
    if twice {
        assert_ne!(calls[7].stdout, b"reset\n", "second immediate cut missed the reset");
        eprintln!(
            "second kill: immediate-cut fallback after sending a fresh keyed reset; reset phase and resumption not shown"
        );
    }
    let tail = &calls[calls.len() - 4..];
    assert_eq!(tail[0].stdout, b"refused: not offered\nreset-read exit 2\n", "not_started under a new reset key");
    for call in tail {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
        assert!(call.stderr.is_empty(), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert_empty_forges(&tail[1]);
    assert_reset_accounts(&tail[2]);
    playground::assert_playground_project(&tail[3], false);
    for path in [format!("box/groups/{forge}"), format!("box/started/{forge}"), format!("forge-{forge}")] {
        assert!(!paths[1].join(&path).exists(), "reset removed {path}");
    }
    eprintln!(
        "not_started; no forge, scripted account, Playground project, group, started marker or forge directory; box exit 0 with nothing left"
    );
    scratch.remove();
}

/// Each poll is a new ctl process and a new key; a successful reset is never replayed.
fn read_not_started(client: &Path) -> Call {
    shell_read(
        client,
        r#""$1" ctl "$2" playground reset --key "reset-cut-read-$$" </dev/null 2>&1
status=$?
case "$status" in
    0) exit 0 ;;
    2) printf 'reset-read exit 2\n' ;;
    *) exit "$status" ;;
esac"#,
    )
}

fn shell_read(client: &Path, script: &str) -> Call {
    Call::command(
        Path::new("/bin/sh"),
        &[
            "-c",
            script,
            "read-reset",
            skein_scratch::paths::binary("temper").to_str().expect("temper path"),
            client.to_str().expect("client path"),
        ],
    )
}

fn assert_empty_forges(call: &Ended) {
    let document = ctl::json(call);
    assert!(document["forges"].as_array().expect("forge rows").is_empty(), "{document}");
}

fn assert_reset_accounts(call: &Ended) {
    let document = ctl::json(call);
    for holder in ["deployment", "party"] {
        assert!(
            !document["accounts"][holder].as_array().expect("account rows").iter().any(|row| row["provider"] == 3),
            "no scripted account: {document}"
        );
    }
}
