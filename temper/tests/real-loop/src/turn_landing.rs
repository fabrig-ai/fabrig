//! A ctl-released batch on the fake Forgejo (testing/qa/setups.md, 5–6).
use crate::{ForgeRecipe, Ran, argv, calls, fixtures, forge, landing, llm};
use serde_json::{Value, json};
use skein_fake_forge_domain::api;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub const A: &str = "@turn-a First turn member\nPerson: 1; Repository: 4294967298;";
pub const B: &str = "@turn-b Second turn member\nPerson: 1; Repository: 4294967298;";
pub const RED: &str = "@turn-red Failing turn member\nPerson: 1; Repository: 4294967298;";
// Each goal is started only after the preceding review exists, so these are
// checked identities of this story, not numbers embedded in the copied scripts.
pub const LATCH: u64 = 2;
pub const CHANGES: [u64; 2] = [3, 7];
pub const REVIEWS: [u64; 2] = [5, 9];
pub const BEFORE: [usize; 2] = [15, 18];
pub const RELEASE: usize = 22;

#[expect(clippy::too_many_lines, reason = "one fixture declares the gate and its prepare carriers together")]
fn fixture() -> forge::Fixture {
    forge::Fixture {
        manual_merges: api::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true },
        setup: || {
            let mut setup = (forge::fixture().setup)();
            setup.workflows = Box::new([api::Workflow {
                file: b"gate.yml".as_slice().into(),
                jobs: Box::new([b"gate".as_slice().into()]),
                requirements: Box::new([]),
                report: None,
                select: None,
                fails: Box::new([]),
                flaky: Box::new([]),
                latency_min: skein_lib::Duration::ZERO,
                latency_max: skein_lib::Duration::ZERO,
                passes: 1000,
                silent: 0,
                cue: Some(api::Cue::Declared { path: b".fake-forge/gate".as_slice().into() }),
            }]);
            setup
        },
        configure: |value, agent, address| {
            (landing::fixture().configure)(value, agent, address);
            fixtures::own_policy(value);
            value["world_settings"]["landing"]["projects"][0]["rules"][0]["gates"][0]["freshness"] = "clean".into();
            // The deployment's stock grants include turn's dispatch, statuses and
            // branch cleanup as well as both producers' write authorities.
            let kinds = [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17];
            let grants: Vec<_> = kinds
                .into_iter()
                .map(|kind| {
                    json!({
                        "connector":1,"kind":kind,"pattern":{"segments":["forge"],"last":"open","value":""}
                    })
                })
                .collect();
            for project in value["world_settings"]["policies"].as_array_mut().expect("policies") {
                let policy = &mut project["policy"];
                policy["period_spend"] = 10000.into();
                policy["roles"][0]["period_spend"] = 8000.into();
                policy["ceiling"]["grants"] = json!(grants);
                policy["ceiling"]["tools"] = 391.into();
                policy["ceiling"]["budget"]["spend"] = 10000.into();
                policy["ceiling"]["delegation"]["kinds"] = json!([
                    {"kind":"charter","number":1},{"kind":"charter","number":2},
                    {"kind":"charter","number":4},{"kind":"procedure","number":2},{"kind":"role","number":0},{"kind":"job","number":1}]);
                policy["roles"][0]["authority"] = policy["ceiling"].clone();
                policy["roles"][0]["authority"]["budget"]["spend"] = 4000.into();
                policy["projections"] = json!(grants);
            }
            value["world_settings"]["kinds"] = json!({"items":[
                {"setting":{"kind":"project-landing","project":1},"set":{"by":"turn","batch":2}},
                {"setting":{"kind":"project-gates","project":1},"set":{"repositories":[
                    {"forge":1,"repository":2,"gates":[{"name":"gate","workflow":"gate.yml","ignored":[],"blocking":true}]}
                ]}}
            ]});
            let limits = &mut value["application"]["limits"]["overrides"];
            for (name, millis) in
                [("poll_ms", 100), ("poll_max_ms", 500), ("slow_ms", 1000), ("hinted_ms", 100), ("fallback_ms", 1000)]
            {
                limits[format!("forge.client.{name}")] = millis.into();
            }
            limits["forge.landing_members"] = 2.into();
            limits["forge.resources_per_task"] = 3.into();
            limits["forge.holds"] = 16.into();

            limits["root.loads.loads"] = 3.into();
            // Both changes, their owners, latch, reviews, landing and prepare job.
            for (name, size) in [
                ("tasks.tasks", 24),
                ("tasks.batch", 2),
                ("authority.batch", 2),
                ("tasks.project_tasks", 24),
                ("tasks.tree_tasks", 24),
                ("tasks.project_goals", 2),
                ("tasks.authority_grants", 16),
                ("tasks.inbox_bytes", 65536),
                ("tasks.inbox_messages", 32),
                ("people.inbox_recipients", 69),
                ("documents.lines", 32),
                ("documents.chips", 32),
                ("authority.executors", 22),
                ("tasks.executor_kinds", 22),
                ("fleet.slots", 2),
                ("fleet.attempts", 2),
                ("core.load_bytes", 131_072),
                ("people.inbox_tasks", 48),
            ] {
                value["engine"]["limits"]["overrides"][name] = size.into();
            }
            value["workers"][0]["slots"] = 2.into();
            value["workers"][0]["limits"]["overrides"]["hub.slots"] = 2.into();
            for limits in ["engine", "workers"] {
                let target = if limits == "engine" { &mut value["engine"] } else { &mut value["workers"][0] };
                target["limits"]["overrides"]["core.call_answer_bytes"] = 8192.into();
            }
            // The shared prepare carriers, rather than the small chat profile.
            let prepare = temper_service::limits::prepare_bound();
            for (name, bytes) in [
                ("tasks.spec_bytes", prepare.spec),
                ("tasks.result_bytes", prepare.result),
                ("tasks.message_bytes", prepare.result),
            ] {
                value["engine"]["limits"]["overrides"][name] = bytes.into();
            }
            value["engine"]["limits"]["overrides"]["hub.outcome_bytes"] = (prepare.result + 2).into();
            value["workers"][0]["limits"]["overrides"]["tasks.result_bytes"] = prepare.result.into();
            value["workers"][0]["limits"]["overrides"]["hub.outcome_bytes"] = (prepare.result + 2).into();
            value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = prepare.result.into();
            value["application"]["limits"]["overrides"]["checkout.message_bytes"] = (prepare.result + 2).into();
            value["application"]["limits"]["overrides"]["forge.client.op_bytes"] = prepare.result.into();
            agent["profile"]["declared"]["tool_payload"] = (prepare.result + 2).into();
            fixtures::turn_bounds::apply(value, fixtures::turn_bounds::turn_body(agent));
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

fn recipe(_: SocketAddr, _: &Path) -> llm::Recipe {
    let base = serde_json::to_vec(&json!({
        "listen":{"socket":"127.0.0.1:0"},"key":"/fixture/model.key","tls":null,
        "models":[{"name":"fake","window":1_048_576,"output":8192}],
        "cued":[],"asked":[],"otherwise":[]
    }))
    .expect("model document");
    let document = temper_fake::llm::Document::decode_with_scripts(
        &base,
        Some(include_bytes!("../tests/fixtures/turn-landing/scripts.json")),
    )
    .expect("box --scripts fixture");
    let mut recipe = llm::scripted(Box::new([]));
    recipe.domain_limits = document.limits();
    recipe.domain = document.domain(fixtures::SEED);
    recipe
}

#[must_use]
pub fn run(red: bool) -> Ran {
    crate::run_forge_with(fixture(), ForgeRecipe { source: PathBuf::new(), build: recipe }, || acts(red))
}

#[expect(clippy::too_many_lines, reason = "the ordered ctl script keeps its observation indices beside both endings")]
fn acts(red: bool) -> Vec<calls::Step> {
    use calls::{D1, call, number, sign_in, until};
    let mut acts = vec![
        sign_in(D1),
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
                "adopt"
            ],
        ),
        call(
            D1,
            argv![
                "goal",
                "start",
                "--project",
                "1",
                "--charter",
                "1",
                "--budget",
                "500",
                "--deadline",
                "2030-01-01T00:00Z",
                "--key",
                "first",
                if red { RED } else { A }
            ],
        ),
        call(D1, argv!["task", "show", "1", "--until", "phase:idle", "--json"]),
        call(D1, argv!["change", "show", "3", "--until", "pull", "--json"]),
        until(D1, argv!["task", "show", "3", "--tree", "--json"], b"\"task\":5", b"\"task\""),
        call(D1, argv!["task", "show", number(LATCH), "--json"]),
        call(
            D1,
            argv![
                "task",
                "amend",
                "3",
                "--rev",
                "0",
                "--dependency",
                number(LATCH),
                "Wait for the batch latch",
                "--key",
                "keep-a"
            ],
        ),
        call(
            D1,
            argv![
                "goal",
                "start",
                "--project",
                "1",
                "--charter",
                "1",
                "--budget",
                "500",
                "--deadline",
                "2030-01-01T00:00Z",
                "--key",
                "second",
                B
            ],
        ),
        call(D1, argv!["task", "show", "6", "--until", "phase:idle", "--json"]),
        call(D1, argv!["change", "show", "7", "--until", "pull", "--json"]),
        until(D1, argv!["task", "show", "7", "--tree", "--json"], b"\"task\":9", b"\"task\""),
        call(
            D1,
            argv![
                "task",
                "amend",
                "7",
                "--rev",
                "0",
                "--dependency",
                number(LATCH),
                "Wait for the batch latch",
                "--key",
                "keep-b"
            ],
        ),
        call(D1, argv!["task", "message", number(REVIEWS[0]), "Approve now", "--key", "approve-a"]),
        call(D1, argv!["task", "show", "5", "--until", "ended", "--json"]),
        until(D1, argv!["change", "show", "3", "--json"], b"\"verdict\":\"approved\"", b"\"step\""),
        call(D1, argv!["task", "message", number(REVIEWS[1]), "Approve now", "--key", "approve-b"]),
        call(D1, argv!["task", "show", "9", "--until", "ended", "--json"]),
        until(D1, argv!["change", "show", "7", "--json"], b"\"verdict\":\"approved\"", b"\"step\""),
        call(D1, argv!["task", "show", "3", "--json"]),
        call(D1, argv!["task", "show", "7", "--json"]),
        call(D1, argv!["landing", "list", "--project", "1", "--ended", "--json"]),
        call(D1, argv!["task", "answer", number(LATCH), "--rev", "0", "Ready together", "--key", "release"]),
    ];
    if red {
        acts.push(until(
            D1,
            argv!["landing", "history", "--project", "1", "--landing", "10", "--json"],
            b"\"kind\":\"gate-ended\"",
            b"\"events\"",
        ));
        // End at the first gate receipt, not a claim about subsequent narrowing or repair.
        for goal in [1, 6, 10] {
            acts.push(call(
                D1,
                argv![
                    "task",
                    "cancel",
                    number(goal),
                    "Stop after the red proof",
                    "--key",
                    match goal {
                        1 => "stop-a",
                        6 => "stop-b",
                        _ => "stop-landing",
                    }
                ],
            ));
            acts.push(call(D1, argv!["task", "show", number(goal), "--until", "ended"]));
        }
    } else {
        for change in CHANGES {
            acts.push(call(D1, argv!["change", "show", number(change), "--until", "landed", "--json"]));
        }
        for goal in [1, 6] {
            acts.push(call(D1, argv!["task", "show", number(goal), "--until", "ended", "--json"]));
        }
    }
    acts.push(call(D1, argv!["landing", "list", "--project", "1", "--ended", "--json"]));
    acts.push(call(D1, argv!["landing", "show", "10", "--json"]));
    acts.into_iter()
        .enumerate()
        .map(|(at, step)| {
            if matches!(step, calls::Step::Call { .. })
                && (!red || ![RELEASE + 2, RELEASE + 4, RELEASE + 6].contains(&at))
            {
                step.succeeds()
            } else {
                step
            }
        })
        .collect()
}

#[must_use]
pub fn document(ran: &Ran, at: usize) -> Value {
    let value: Value = serde_json::from_slice(&ran.calls()[at].stdout).expect("ctl JSON");
    value
        .get("task")
        .filter(|task| task.is_object())
        .or_else(|| value.get("change").filter(|change| change.is_object()))
        .unwrap_or(&value)
        .clone()
}
