//! A failed GitHub check is repaired at a new head (domain/testing.md, 2.4).
use serde_json::Value;
use skein_fake_forge_domain::{Observation, api};
use skein_fake_llm_domain::api::Scripts;
use temper_real_loop_world::{
    Exit, argv,
    calls::{D1, Step, call, file, number, sign_in, until},
    fixtures,
    github::{self, check_chat_heard_the_landing_at, check_landing_at},
    landing, llm, run_github,
};

const ONE_MERGE: usize = 1;
const TWO_PUSHES: usize = 2;

const DIAGNOSTIC: &[u8] = b"Missing changes/40.md with the line: opened 2026-11-02T14:01:43Z";
const REQUIRED: &str = "opened 2026-11-02T14:01:43Z\n";
const REPAIR_CUE: &[u8] = b"Repair the failed CI on the change branch";
// Producer, five repair turns and the review fit within sixteen units.
// The review keeps B2's four-unit floor.
const REPAIR_CHANGE_SPEND: u64 = 16;
// Jig's delegate and forge-read families (domain/agent.md, 4.3).
const REPAIR_TOOLS: u64 = 513;

fn repair_log() -> Box<[u8]> {
    const LOG_BYTES: usize = 4_096;
    let mut log = vec![b'\n'; LOG_BYTES - 2_200];
    log.extend_from_slice(DIAGNOSTIC);
    log.extend_from_slice(b"\n2026-11-02T14:01:45Z ##[error]Process completed with exit code 1.\n");
    log.extend_from_slice(b"2026-11-02T14:01:46Z Post job cleanup.\n");
    while log.len() < LOG_BYTES {
        log.extend_from_slice(b"git config --local --get-regexp http.extraheader\n");
    }
    log.truncate(LOG_BYTES);
    log.into_boxed_slice()
}

fn repair_setup() -> api::Setup {
    let mut setup = (temper_real_loop_world::forge::fixture().setup)();
    setup.tree =
        [("README.md", "# app\n"), ("src/main.rs", "fn main() {}\n"), (".github/workflows/ci.yml", "name: changes\n")]
            .map(|(path, content)| api::File { path: path.as_bytes().into(), content: content.as_bytes().into() })
            .into();
    setup.checks.cue = Some(api::Cue::PullFile {
        path: b"changes/40.md".as_slice().into(),
        no_pull: api::Check::Passed,
        step: b"changes".as_slice().into(),
        log: repair_log(),
    });
    assert!(
        setup
            .tree
            .iter()
            .all(|file| !file.content.windows(REQUIRED.trim().len()).any(|part| part == REQUIRED.trim().as_bytes())),
        "the timestamp is absent from the checkout"
    );
    setup
}

fn repair_fixture() -> github::Fixture {
    let mut fixture = github::fixture();
    fixture.setup = repair_setup;
    fixture.configure_domain = |config| {
        // Keep CI for both main and the change, with no mergeability delay.
        config.limits.content_bytes = 4_096;
        config.limits.statuses = 2;
        config.limits.observations = 32;
        config.settle_min = skein_lib::Duration::ZERO;
        config.settle_max = skein_lib::Duration::ZERO;
    };
    fixture.prepare = |forge, env| {
        // Seed a green main through the same content cue.
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
        for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
            project["policy"]["ceiling"]["tools"] = REPAIR_TOOLS.into();
            project["policy"]["roles"][0]["authority"]["tools"] = REPAIR_TOOLS.into();
        }
        // Keep the shipped brief and model limits; only polling is accelerated.
        let limits = &mut value["application"]["limits"]["overrides"];
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
    check_repaired_head(records);
    assert_eq!(ran.github().git_calls.iter().filter(|call| call.writes).count(), TWO_PUSHES);
    assert!(
        ran.llm().iter().any(|seen| matches!(seen,
            skein_fake_peers::llm::Observation::Query { query, .. }
            if skein_lib::bytes::find(&query.system, REPAIR_CUE).is_some()
                && skein_lib::bytes::find(&query.system, DIAGNOSTIC).is_some()
        )),
        "the repair model request contains the complete gate diagnostic"
    );
    check_job_read(&ran);
    for (index, call) in ran.calls().iter().enumerate() {
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {index}: {call:?}");
    }
    let main = ran.github().branches.get(b"main".as_slice()).expect("landed main");
    assert_eq!(main.tree.get(b"changes/40.md".as_slice()).expect("repair file"), REQUIRED.as_bytes());
    let commit = check_landing_at(&ran, CHANGE_READ);
    check_chat_heard_the_landing_at(&ran, &commit, CHANGE_READ, CHAT_READ);
}

fn check_job_read(ran: &temper_real_loop_world::Ran) {
    assert!(
        ran.llm().iter().any(|seen| {
            let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { return false };
            query.messages.iter().flat_map(|message| message.parts.iter()).any(|part| {
                let skein_fake_llm_domain::api::Part::ToolOutput { output: content, is_error: false, .. } = part else {
                    return false;
                };
                content.windows(DIAGNOSTIC.len()).any(|line| line == DIAGNOSTIC)
            })
        }),
        "the repair reads the diagnostic through read_job before writing"
    );
}

fn check_repaired_head(records: &[Observation]) {
    let heads: Vec<_> = records
        .iter()
        .filter_map(|row| {
            let Observation::Moved { repository, branch, to, .. } = row else { return None };
            (repository.as_ref() == b"acme/app" && branch.as_ref() != b"main").then_some(*to)
        })
        .collect();
    let [first_head, repaired_head] = heads.as_slice() else { panic!("two pushed change heads: {heads:?}") };
    let failed_at = records
        .iter()
        .position(|row| {
            matches!(row,
                Observation::Reported { commit, state: api::Check::Failed, .. } if commit == first_head
            )
        })
        .expect("pull-specific CI failed");
    let reports: Vec<_> = records
        .iter()
        .enumerate()
        .skip(failed_at.saturating_sub(1))
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
    check_merge(records, *b, *passed);
}

fn check_merge(records: &[Observation], head: u64, passed: usize) {
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let Observation::Merged { head, .. } = row else { return None };
            Some((at, *head))
        })
        .collect();
    assert_eq!(merges.len(), ONE_MERGE, "the repaired head must merge once");
    assert_eq!(merges[0].1, head, "only the repaired, passing head merged");
    assert!(passed < merges[0].0, "CI passed before merge");
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

fn failed_head() -> String {
    // The two repositories start at 1 and 2; seeded main is 3; the producer makes 4.
    use std::fmt::Write;
    let mut head = String::new();
    for byte in skein_fake_forge_domain::object_bytes(4) {
        write!(head, "{byte:02x}").expect("commit hex");
    }
    head
}

fn repair_scripts(at: std::net::SocketAddr) -> Scripts {
    use serde_json::json;
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let mut scripts = landing::scripts(at, u64::from(github::REPOSITORY), landing::README).into_vec();
    let Line::Call { arguments, .. } = &mut scripts[0].turns[0].lines[0] else { panic!("change delegation") };
    let mut delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    delegate["batch"][0]["authority"]["budget"]["spend"] = REPAIR_CHANGE_SPEND.into();
    delegate["batch"][0]["authority"]["tools"] = REPAIR_TOOLS.into();
    let grants = delegate["batch"][0]["authority"]["grants"].as_array_mut().expect("grants");
    let mut read_grant = grants[0].clone();
    read_grant["kind"] = 1.into();
    grants.push(read_grant);
    *arguments = serde_json::to_vec(&delegate).expect("delegate JSON").into();
    let call = |name: &[u8], arguments: Value| Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(&arguments).expect("repair JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    scripts.push(Script {
        // The repair can run only after the model receives the gate's timestamp.
        cue: DIAGNOSTIC.into(),
        turns: Box::new([
            call(
                b"read_job",
                json!({"repository":"acme/app", "head": failed_head(),
                "run":3, "job":3, "attempt":1, "max_bytes":1024}),
            ),
            call(b"read", json!({"path":"src/main.rs"})),
            call(b"read", json!({"path":".github/workflows/ci.yml"})),
            call(b"write", json!({"path":"changes/40.md","content":REQUIRED})),
            call(
                b"finish",
                json!({"form":"change","fields":{"title":"Supply the gate timestamp","body":"Repair CI."}}),
            ),
        ]),
    });
    llm::cued(scripts.into_boxed_slice())
}

fn refused_repair_scripts(at: std::net::SocketAddr) -> Scripts {
    use serde_json::json;
    use skein_fake_llm_domain::api::{Finish, Line, Turn};
    let mut scripts = repair_scripts(at);
    // Each refused repair wakes the chat with a hold, rather than a landed change.
    let chat = scripts.cued.first_mut().expect("chat script");
    let mut turns = chat.turns.to_vec();
    let waiting = Turn {
        lines: Box::new([
            Line::Text { text: b"The repair is held.".as_slice().into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    *turns.last_mut().expect("chat's landing report") = waiting.clone();
    turns.push(waiting);
    chat.turns = turns.into();
    let script = scripts.cued.last_mut().expect("repair script");
    script.turns = Box::new([Turn {
        lines: Box::new([Line::Call {
            name: b"finish".as_slice().into(),
            arguments: serde_json::to_vec(&json!({"form":"verdict","label":"no_change",
                "text":"Declining this repair for the release regression.","fields":{},"items":[]}))
            .expect("no-change verdict JSON")
            .into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }]);
    scripts
}

fn replacement_acts() -> Vec<Step> {
    let mut acts = repair_acts();
    acts.truncate(6);
    acts.extend([
        call(D1, argv!["change", "show", number(CHANGE_TASK), "--until", "held", "--json"]),
        call(D1, argv!["task", "release", number(CHANGE_TASK), "--hold", "1"]),
        // The replacement's no_change verdict creates hold 2; the panel can still show hold 1 after release.
        until(
            D1,
            argv!["change", "show", number(CHANGE_TASK), "--until", "held", "--json"],
            b"task release 2 --hold 2;",
            b"task release 2 --hold 1;",
        ),
        call(D1, argv!["task", "activity", number(CHANGE_TASK), "--json"]),
        // The replacement's hold wakes the chat; wait for its acknowledgement to park.
        call(D1, argv!["task", "show", number(CHAT_TASK), "--until", "phase:idle", "--json"]),
    ]);
    acts
}

#[test]
fn a_replacement_github_repair_after_release_receives_the_job_log_again() {
    let ran = run_github(repair_fixture(), refused_repair_scripts, replacement_acts);
    let prompts: Vec<_> = ran
        .llm()
        .iter()
        .filter_map(|seen| {
            let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { return None };
            skein_lib::bytes::find(&query.system, REPAIR_CUE).is_some().then_some(&query.system)
        })
        .collect();
    assert_eq!(prompts.len(), 2, "first repair and replacement after release");
    for prompt in prompts {
        assert!(skein_lib::bytes::find(prompt, DIAGNOSTIC).is_some());
    }
    assert!(!ran.github().observations().iter().any(|row| matches!(row, Observation::Merged { .. })));
    for (index, call) in ran.calls().iter().enumerate() {
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {index}: {call:?}");
    }
}
