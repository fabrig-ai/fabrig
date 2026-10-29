//! QA-B: connected GitHub after pull creation, on the shipped real loop.
//! Design, public harness surface and explicitly allowed usage examples only.
use serde_json::Value;
use skein_fake_forge_domain::{Observation, api};
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use std::{fmt::Write, net::SocketAddr};
use temper_real_loop_world::{
    Exit, Ran, argv,
    calls::{self, D1, Step, call, file, number, sign_in, task},
    github::{self, check_chat_heard_the_landing, check_connection, check_landing, connect_adopt_and_ask_for_a_change},
    landing, run_github, terminal,
};

const CHANGE: u64 = 2;
const CHAT_CALL: usize = 4;
const CHAT: u64 = 1;
const LANDING_HEARD: usize = 8;
const ONE: usize = 1;
const LANDED: &[u8] = b"The change landed.";

fn fixture() -> github::Fixture {
    github::Fixture {
        configure: |value, agent, address| {
            // Keep the ordinary enforced CI, exact review and up-to-date rule.
            (landing::fixture().configure)(value, agent, address);
            value["engine"]["limits"]["overrides"]["documents.rows"] = 3.into();
            temper_real_loop_world::fixtures::requester(value);
            // The generic fixture's chat is unlisted; these stories follow it on the person's page.
            temper_real_loop_world::fixtures::listed(value);
        },
        ..github::fixture()
    }
}

fn scripts(address: SocketAddr) -> Box<[Script]> {
    let mut scripts = landing::scripts(address, u64::from(github::REPOSITORY), landing::README);
    // Keep the person's ordinary requester-ended chat open after its report.
    scripts[0].turns[2] = Turn {
        lines: Box::new([
            Line::Text { text: LANDED.into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    scripts
}

fn begin() -> Vec<Step> {
    vec![
        sign_in(D1),
        call(D1, argv!["account", "add-key", "1", file("fixture.key"), "--deployment", "--key", "qa-llm"]),
        call(D1, argv!["account", "list", "--until", "usable"]),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                "1",
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--key",
                "qa-adopt"
            ],
        ),
        call(D1, argv!["chat", "start", "--project", "1", "--key", "qa-chat", "@forge_chat Make one small edit"]),
    ]
}

fn landed_steps() -> Vec<Step> {
    leave_chat_open(landing_reads())
}

fn landing_reads() -> Vec<Step> {
    let mut steps = begin();
    steps.extend([
        // The chat's committed first reply follows delegate creation. Opening
        // a watch on T2 before that would correctly answer unknown.
        call(D1, argv!["task", "show", task(CHAT_CALL), "--until", "replied"]),
        call(D1, argv!["task", "show", number(CHANGE), "--until", "ended", "--json"]),
        // Ended tasks retain a plain result; --tree is a live watch.
        call(D1, argv!["task", "show", number(CHANGE), "--json"]),
        calls::until(D1, argv!["chat", "show", task(CHAT_CALL), "--json"], LANDED, b"Change requested"),
        call(D1, argv!["task", "show", number(CHANGE), "--json"]),
    ]);
    steps
}

fn leave_chat_open(mut steps: Vec<Step>) -> Vec<Step> {
    // These requester-ended chats remain available on the person's page.
    // Join the activation that published the reply, without closing the chat.
    steps.extend([
        call(D1, argv!["task", "show", number(CHAT), "--until", "phase:idle", "--json"]),
        calls::ends_leaving(&[temper_real_loop_world::ending::Leftover {
            task: CHAT,
            may_be: &[temper_real_loop_world::ending::Left::Due],
            why: "the requester-ended chat remains open after its joined reply for the person to follow on the page",
        }]),
    ]);
    steps
}

fn succeeded(ran: &Ran) {
    for (index, result) in ran.calls().iter().enumerate() {
        assert_eq!(
            result.exit,
            Some(Exit::Code(0)),
            "call {index}: {} {}; pull observations {:?}; branch heads {:?}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
            ran.github().pulls,
            ran.github()
                .branches
                .iter()
                .map(|(name, branch)| (String::from_utf8_lossy(name), branch.head))
                .collect::<Vec<_>>()
        );
    }
}

fn document(ran: &Ran, at: usize) -> Value {
    serde_json::from_slice(&ran.calls()[at].stdout).expect("client JSON document")
}

fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    calls::occurrences(bytes, needle) > 0
}

fn landed(ran: &Ran) -> u64 {
    succeeded(ran);
    let opened = ran.github().pulls.iter().filter(|seen| matches!(seen, Observation::Opened { .. })).count();
    let merged: Vec<_> = ran
        .github()
        .pulls
        .iter()
        .filter_map(
            |seen| if let Observation::Merged { head, commit, .. } = seen { Some((*head, *commit)) } else { None },
        )
        .collect();
    assert_eq!(opened, 1, "one pull through repeated client reads: {:?}", ran.github().pulls);
    assert_eq!(merged.len(), ONE, "one merge: {:?}", ran.github().pulls);
    let (head, commit) = merged[0];
    let topic: Vec<_> = ran.github().branches.iter().filter(|(name, _)| name.as_slice() != b"main").collect();
    assert_eq!(topic.len(), ONE, "one worker branch: {:?}", ran.github().branches);
    assert_eq!(topic[0].1.head, head, "merge exactly the worker's pushed head");
    let main = ran.github().branches.get(b"main".as_slice()).expect("observed main head after merge");
    assert_eq!(main.head, commit, "main is the observed merge commit");
    assert_eq!(main.tree.get(b"README.md".as_slice()), Some(&[landing::README, landing::EDIT].concat()));
    let mut sha = String::new();
    for byte in skein_fake_forge_domain::object_bytes(main.head) {
        write!(sha, "{byte:02x}").expect("format commit id");
    }
    let task = document(ran, 7);
    assert_eq!(task["chip"]["task"], CHANGE);
    assert_eq!(task["chip"]["phase"], "ended");
    let result = &task["result"]["value"]["change"];
    assert_eq!(result["kind"], 1, "landed change");
    assert_eq!(
        result["words"]["bytes"].as_str().expect("landing words").lines().next(),
        Some(sha.as_str()),
        "first result line names main's merge commit"
    );
    for request in &ran.github().git_calls {
        assert!(request.status == 200 || (!request.writes && request.status == 404), "authenticated git: {request:?}");
        assert_eq!(request.minted.expect("installation token").installation, github::INSTALLATION);
    }
    assert!(ran.github().git_calls.iter().any(|request| request.writes));
    head
}

#[test]
fn checked_head_lands_once_and_repeated_reads_name_main() {
    // QA: domain/forge.md 8.2, 8.7, 8.8; domain/testing.md 2.4 story 1.
    let ran = run_github(github::Fixture { keep_heads: true, ..fixture() }, scripts, landed_steps);
    landed(&ran);
    let before = document(&ran, 7);
    let after = document(&ran, 9);
    let before = before.get("result").expect("first result field");
    let after = after.get("result").expect("repeated result field");
    assert!(!before.is_null(), "landed task has a result");
    assert_eq!(before, after, "terminal result survives repeated reads");
    assert!(contains(&ran.calls()[LANDING_HEARD].stdout, LANDED), "the chat hears the landing before replying");
    assert!(
        ran.github().app_calls.iter().any(|(_, path)| path.starts_with(b"/app/hook/deliveries")),
        "real puller must poll its delivery log"
    );
}

fn page_steps() -> Vec<Step> {
    let mut steps = landing_reads();
    steps.extend([
        terminal::open(),
        // client/README.md 3.1, 5.1: this person starts in project 1.
        // Wait for its selected control before opening the project's chats.
        terminal::see(b"Project 1 (pressed)"),
        terminal::see(b"Chats"),
        terminal::type_on(b"Chats", b""),
        terminal::see(b"T1"),
        terminal::type_on(b"T1", b""),
        terminal::see(LANDED),
        terminal::close(),
    ]);
    leave_chat_open(steps)
}

#[test]
fn landed_chat_is_readable_on_the_persons_page() {
    // QA: domain/testing.md 2.4 story 1; domain/forge.md 8.2, 8.8.
    let ran = run_github(fixture(), scripts, page_steps);
    landed(&ran);
    assert_eq!(ran.terminal().exit, Some(Exit::Code(0)));
    assert!(ran.terminal().pages.iter().any(|page| contains(&page.bytes, LANDED)));
}

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
    fixture.connected = false;
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
        limits["forge.client.poll_ms"] = 30.into();
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
    check_connection(&ran);
    let commit = check_landing(&ran);
    check_chat_heard_the_landing(&ran, &commit);
}

fn repair_acts() -> Vec<Step> {
    let mut steps = connect_adopt_and_ask_for_a_change();
    let wait = steps.pop().expect("final chat join after the landing");
    steps.push(wait.timeout(4));
    steps
}

fn repair_scripts(at: std::net::SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]> {
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
    scripts.into_boxed_slice()
}
