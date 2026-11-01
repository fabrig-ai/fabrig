//! QA pass 20261027-0037: context leaves the writer alone; setup keys replay.
use super::{Call, Walk, boxed, ctl};
use skein_io::kernel::Exit;

fn adopt(forge: u16, project: &str, role: &str, key: &str) -> Call {
    Call::new(&[
        "repository",
        "adopt",
        "--project",
        project,
        "--forge",
        &forge.to_string(),
        "--owner",
        "playground",
        "--name",
        "widget",
        "--landing",
        "main",
        "--role",
        role,
        "--key",
        key,
        "--json",
    ])
}

fn setup(key: &'static str) -> ctl::Step {
    ctl::Step::Then(Box::new(move |seen| {
        let number = super::forges::made(&seen[1]);
        Call::new(&["forge", "setup", &number.to_string(), "--playground", "--key", key, "--json"])
    }))
}

fn adoption(project: &'static str, role: &'static str, key: &'static str) -> ctl::Step {
    ctl::Step::Then(Box::new(move |seen| adopt(super::forges::made(&seen[1]), project, role, key)))
}

#[test]
fn context_preserves_the_writer_and_a_refused_setup_preserves_the_first_key() {
    let walk = Walk::owned("walk-context-and-setup-replay");
    let outcome = boxed::run_ctl_with_exits(
        walk.scratch.path(),
        super::super::flags(&walk.paths[0], &walk.paths[1], &walk.paths[2], Some(&walk.bundle)),
        walk.scratch.path(),
        vec![
            ctl::Step::Run(Call::sign_in()),
            ctl::Step::Run(super::forges::make_here()),
            setup("first"),
            setup("first"),
            adoption("1", "owned", "other"),
            adoption("1", "context", "reader"),
            adoption("2", "owned", "writer"),
            ctl::Step::Run(Call::new(&["forge", "list", "--json"])),
            setup("second"),
            setup("first"),
            ctl::Step::Run(Call::new(&["forge", "list", "--json"])),
        ],
        crate::BOX_DEADLINE,
        || (),
        &[(4, 2), (8, 2)],
    );
    let diagnostic = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "box drained: {diagnostic}");
    assert_eq!(outcome.left, Some(Vec::new()), "no child outlives its box: {diagnostic}");
    let [signed, made, first, replay, other, reader, writer, before, refused, replay_after, after] =
        outcome.calls.as_slice()
    else {
        panic!("all eleven ctl calls completed: {diagnostic}")
    };
    for call in [signed, made, first, replay, reader, writer, before, replay_after, after] {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "{} {}; box: {diagnostic}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&other.stderr).expect("refusal JSON"),
        serde_json::json!({"forge_refused":{"reason":"another_project"}})
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&refused.stderr).expect("refusal JSON"),
        serde_json::json!({"refused":{"why":"not_offered"}})
    );
    assert_eq!(other.exit, Some(Exit::Code(2)));
    assert_eq!(refused.exit, Some(Exit::Code(2)));
    let answers = serde_json::Deserializer::from_slice(&first.stdout)
        .into_iter::<serde_json::Value>()
        .collect::<Result<Vec<_>, _>>()
        .expect("setup JSON");
    assert_eq!(answers.len(), 2, "progress and adoption");
    assert_eq!(answers[1]["project"], 2, "setup made the writer's project");
    assert_eq!(answers[0], serde_json::json!({"setting_up":{}}));
    assert_eq!(answers[1]["outcome"]["adopted"]["owner"], "playground");
    assert_eq!(answers[1]["outcome"]["adopted"]["name"], "widget");
    assert_eq!(answers[1]["outcome"]["adopted"]["home"], true);
    assert_eq!(first.stdout, replay.stdout, "first key replays before the refusal");
    assert_eq!(first.stdout, replay_after.stdout, "first key replays after the refusal");
    let context = ctl::json(reader);
    assert_eq!(context["adopted"]["role"], "context");
    assert_eq!(context["adopted"]["home"], false);
    let owned = ctl::json(writer);
    assert_eq!(owned["adopted"]["role"], "owned");
    assert_eq!(owned["adopted"]["home"], true);
    assert_eq!(context["adopted"]["repository"], owned["adopted"]["repository"]);
    assert_eq!(ctl::json(before), ctl::json(after), "setup refusal/replay leaves forge and first repository unchanged");
    walk.scratch.remove();
}
