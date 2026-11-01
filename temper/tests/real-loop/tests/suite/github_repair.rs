//! A failed GitHub check is repaired at a new head (domain/testing.md, 2.4).
use serde_json::Value;
use skein_fake_forge_domain::{Observation, api};
use skein_fake_llm_domain::api::Scripts;
use temper_real_loop_world::{
    Exit, argv,
    calls::{D1, Step, call, file, number, sign_in},
    fixtures,
    github::{self, check_chat_heard_the_landing_at, check_landing_at},
    landing, llm, run_github,
};

const ONE_MERGE: usize = 1;
const TWO_PUSHES: usize = 2;

// Main starts green; the coder removes its heading, and the repair restores it.
const BROKEN: &str = "Broken by temper.\n";
const REPAIR_CUE: &[u8] = b"Repair the failed CI on the change branch";
// Producer and repair spend six units; four left holds the review at B2's floor.
// Six plus working room above that floor fits in the next power of two, sixteen.
const REPAIR_CHANGE_SPEND: u64 = 16;

fn repair_fixture() -> github::Fixture {
    let mut fixture = github::fixture();
    fixture.setup = || {
        let mut setup = (temper_real_loop_world::forge::fixture().setup)();
        setup.checks.cue = Some(api::Cue::File { path: b"README.md".as_slice().into(), green: landing::README.into() });
        setup
    };
    fixture.configure_domain = |config| {
        // Keep CI for both main and the change, with no mergeability delay.
        config.limits.statuses = 2;
        config.limits.observations = 32;
        config.settle_min = skein_lib::Duration::ZERO;
        config.settle_max = skein_lib::Duration::ZERO;
    };
    fixture.prepare = |forge, env| {
        // Setup starts no CI. Seed a green main through the same content cue.
        skein_fake_forge_domain::advance(
            forge,
            env,
            b"acme/app",
            b"main",
            b"README.md",
            landing::README,
            temper_github_app::APP.bot,
        )
        .expect("main checked before the chat's change");
    };
    fixture.configure = |value, agent, at| {
        (landing::fixture().configure)(value, agent, at);
        // The required CI section carries the failure to the repair worker.
        let limits = &mut value["application"]["limits"]["overrides"];
        for name in ["read_bytes", "budgets.ci", "brief_bytes"] {
            limits[format!("root.brief.{name}")] = 512.into();
        }
        limits["forge.client.poll_ms"] = 10.into();
        limits["forge_component.log_poll_ms"] = 100.into();
    };
    fixture
}

#[test]
fn a_chats_failed_github_check_is_repaired_at_a_new_head_and_lands_once() {
    // QA: domain/forge.md 8.2-8.4, 8.7; domain/testing.md 2.4; Q1.
    let ran = run_github(repair_fixture(), repair_scripts, repair_acts);
    let records = ran.github().observations();
    let seeded_main = records
        .iter()
        .find_map(|row| {
            let Observation::Moved { branch, to, .. } = row else { return None };
            (branch.as_ref() == b"main").then_some(*to)
        })
        .expect("seeded main head");
    assert!(
        records.iter().any(|row| matches!(row,
            Observation::Reported { commit, state: api::Check::Passed, .. } if *commit == seeded_main
        )),
        "main passed its content-cued CI"
    );
    let heads: Vec<_> = records
        .iter()
        .filter_map(|row| {
            let Observation::Moved { repository, branch, to, .. } = row else { return None };
            (repository.as_ref() == b"acme/app" && branch.as_ref() != b"main").then_some(*to)
        })
        .collect();
    let [first_head, repaired_head] = heads.as_slice() else { panic!("two pushed change heads: {heads:?}") };
    let reports: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let Observation::Reported { repository, commit, context, state, .. } = row else { return None };
            (repository.as_ref() == b"acme/app"
                && context.as_ref() == b"check"
                && (commit == first_head || commit == repaired_head))
                .then_some((at, *commit, *state))
        })
        .collect();
    let [
        (pending_a, a, api::Check::Pending),
        (failed, failed_head, api::Check::Failed),
        (pending_b, b, api::Check::Pending),
        (passed, passed_head, api::Check::Passed),
    ] = reports.as_slice()
    else {
        panic!("failed then passed at two heads: {reports:?}")
    };
    assert_eq!(a, first_head);
    assert_eq!(b, repaired_head);
    assert_eq!(a, failed_head);
    assert_eq!(b, passed_head);
    assert_ne!(a, b, "repair pushes a new head");
    assert!(pending_a < failed && failed < pending_b && pending_b < passed);
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let Observation::Merged { head, .. } = row else { return None };
            Some((at, *head))
        })
        .collect();
    assert_eq!(merges.len(), ONE_MERGE, "the repaired head must merge once");
    assert_eq!(merges[0].1, *b, "only the repaired, passing head merged");
    assert!(*passed < merges[0].0, "CI passed before merge");
    assert_eq!(ran.github().git_calls.iter().filter(|call| call.writes).count(), TWO_PUSHES);
    assert!(
        ran.llm().iter().any(|seen| matches!(seen,
            skein_fake_peers::llm::Observation::Query { query, .. }
            if skein_lib::bytes::find(&query.system, REPAIR_CUE).is_some()
        )),
        "the worker ran the engine's repair task"
    );
    for (index, call) in ran.calls().iter().enumerate() {
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {index}: {call:?}");
    }
    let commit = check_landing_at(&ran, CHANGE_READ);
    check_chat_heard_the_landing_at(&ran, &commit, CHANGE_READ, CHAT_READ);
}

// The connected fixture needs no App-connect or installation flow; github_land owns those.
// The single seeded project's first chat delegates the first change.
const CHAT_TASK: u64 = 1;
const CHANGE_TASK: u64 = 2;
const CHANGE_READ: usize = 7;
const CHAT_READ: usize = 8;

fn repair_acts() -> Vec<Step> {
    vec![
        sign_in(D1),
        call(D1, argv!["account", "add-key", "1", file("fixture.key"), "--deployment", "--key", "repair-llm"]),
        call(D1, argv!["account", "list", "--until", "usable"]),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                fixtures::PROJECT,
                "--forge",
                "1",
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--key",
                "repair-adopt"
            ],
        ),
        call(
            D1,
            argv!["chat", "start", "--project", fixtures::PROJECT, "--key", "repair-chat", "@forge_chat Update README"],
        ),
        call(D1, argv!["task", "show", number(CHAT_TASK), "--until", "replied"]),
        call(D1, argv!["task", "show", number(CHANGE_TASK), "--until", "ended", "--json"]),
        call(D1, argv!["task", "show", number(CHANGE_TASK), "--json"]),
        call(D1, argv!["task", "show", number(CHAT_TASK), "--until", "ended", "--json"]).timeout(4),
    ]
}

fn repair_scripts(at: std::net::SocketAddr) -> Scripts {
    use serde_json::json;
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let mut scripts = landing::scripts(at, u64::from(github::REPOSITORY), landing::README).into_vec();
    let Line::Call { arguments, .. } = &mut scripts[0].turns[0].lines[0] else { panic!("change delegation") };
    let mut delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    delegate["batch"][0]["authority"]["budget"]["spend"] = REPAIR_CHANGE_SPEND.into();
    *arguments = serde_json::to_vec(&delegate).expect("delegate JSON").into();
    // The shared exchange delegates enough authority for producer, repair and review.
    scripts[1].turns[1].lines = Box::new([Line::Call {
        name: b"edit".as_slice().into(),
        arguments: serde_json::to_vec(&json!({"path":"README.md","edits":[{"old":"# app\n","new":BROKEN}]}))
            .expect("edit JSON")
            .into(),
    }]);
    let call = |name: &[u8], arguments: Value| Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(&arguments).expect("repair JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    scripts.push(Script {
        cue: REPAIR_CUE.into(),
        turns: Box::new([
            call(b"read", json!({"path":"README.md"})),
            call(b"edit", json!({"path":"README.md","edits":[{"old":BROKEN,"new":"# app\nChanged by temper.\n"}]})),
            call(b"finish", json!({"form":"change","fields":{"title":"Restore README heading","body":"Repair CI."}})),
        ]),
    });
    llm::cued(scripts.into_boxed_slice())
}
