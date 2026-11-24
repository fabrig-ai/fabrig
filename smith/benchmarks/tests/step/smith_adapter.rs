//! Replaying controls (smith/docs/design/testing.md, 2).

use crate::common;
use rendering::{Fixture, LOGIN, authored_task, probe_directory, read_probe};
use skein_fake_machine::Item;
use skein_lib::Duration;
use smith_bench::agent::Context;
use smith_bench::files::{Files, Kind};
use smith_bench::{End, Forced, Measure};
use smith_bench_fixtures::files::Tree;
use std::path::{Path, PathBuf};

mod rendering;
mod retried;

#[test]
fn every_smith_pin_is_a_product_settings_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("agents/smith");
    for name in ["standard", "everything"] {
        let pin = smith_bench::read_configuration(&root.join(format!("{name}.pin.toml"))).expect("pin");
        assert_eq!(pin.pin.configuration, PathBuf::from(format!("{name}.toml")));
        let layer: toml::Value =
            toml::from_str(&std::fs::read_to_string(root.join(&pin.pin.configuration)).expect("TOML layer"))
                .expect("product TOML");
        assert_eq!(layer.as_table().expect("layer").keys().map(String::as_str).collect::<Vec<_>>(), ["trace"]);
        assert_eq!(layer["trace"]["capture"].as_str(), Some(if name == "standard" { "calls" } else { "everything" }));
    }
}

#[test]
fn a_time_breakdown_sums_to_task_wall_within_one_second() {
    let limits = {
        let mut fixture_limits = smith_events_fixtures::SMALL;
        fixture_limits.string = 524_288;
        fixture_limits.content = 4096;
        fixture_limits.items = 512;
        fixture_limits
    };
    let mut observer =
        smith_bench::agent::smith::events::Observer::with_ceiling(&limits, 1_048_576).expect("recorded stream bounds");
    observer.feed(include_bytes!("../recorded/smith-time.jsonl")).expect("recorded smith trace");
    observer.finish().expect("complete trace");
    let mut timing = smith_bench::parse_result(Path::new("fixture"), include_str!("../recorded/results/attempt.json"))
        .expect("result fixture")
        .timing;
    observer.timing(&mut timing);
    let record = &timing.record;
    let value = |measure: &Measure<u64>| match measure {
        Measure::Observed { value } => *value,
        other @ (Measure::LowerBound { .. } | Measure::Unavailable { .. }) => panic!("missing trace span: {other:?}"),
    };
    let sum = value(&record.requests_ms)
        + value(&record.compaction_ms)
        + value(&record.children_ms)
        + value(&record.rest_ms)
        + record.tools_ms.iter().map(|(_, span)| value(span)).sum::<u64>();
    assert!(sum.abs_diff(764_602) < 1000, "recorded task wall: {sum}");
    assert!((380_000..381_000).contains(&value(&record.requests_ms)));
    assert!((93_000..94_000).contains(&value(&record.compaction_ms)));
    assert!((32_000..33_000).contains(&value(&timing.first_byte_ms)));
    assert_eq!(timing.first_byte_ms, record.first_byte_ms);
    assert_eq!(timing.largest_gap_ms, record.largest_gap_ms);
    assert!(value(&timing.longest_completion_ms) > 10_000);
}

#[test]
fn an_exec_arm_classifies_exit_5_as_needs_input() {
    let mut records = common::events::balanced();
    for record in &mut records {
        match &mut record.event {
            smith_events::Event::RunCompleted(run) => {
                run.status = smith_events::Status::Parked;
                run.result = None;
            }
            smith_events::Event::SessionEnded(session) => session.exit = 5,
            smith_events::Event::Budget(_)
            | smith_events::Event::SessionStarted(_)
            | smith_events::Event::RunStarted(_)
            | smith_events::Event::ConversationOpened(_)
            | smith_events::Event::ConversationClosed(_)
            | smith_events::Event::ResponseStarted(_)
            | smith_events::Event::ResponseCompleted(_)
            | smith_events::Event::TextDelta(_)
            | smith_events::Event::ToolStarted(_)
            | smith_events::Event::JobEnded(_)
            | smith_events::Event::ToolCompleted(_)
            | smith_events::Event::CheckStarted(_)
            | smith_events::Event::CheckCompleted(_)
            | smith_events::Event::MessageReceived(_)
            | smith_events::Event::MessageRead(_)
            | smith_events::Event::MessageUnread(_)
            | smith_events::Event::Notice(_) => {}
        }
    }
    let mut observer = smith_bench::agent::smith::events::Observer::new(&common::events::LIMITS).expect("reader");
    observer.feed(&common::events::bytes(&records)).expect("records");
    observer.finish().expect("complete stream");
    assert!(matches!(
        observer.classify(smith_bench::Exit::Code(5), Forced::No).end,
        End::Failed { reason: smith_bench::FailureReason::InputNeeded, .. }
    ));
    assert!(matches!(observer.classify(smith_bench::Exit::Code(0), Forced::No).end, End::HarnessError { .. }));
}

#[test]
fn path_mounts_reach_native_policy_and_escaped_mounts_refuse_before_credentials() {
    let [app, spec] = path_mounts();
    let fixture = Fixture::new(&[
        app,
        spec,
        Item::directory(b"attempt/outside"),
        Item::link(b"attempt/workspace/escaped", b"/attempt/outside"),
        Item::file(b"attempt/workspace/app/input.txt", b"input"),
        Item::directory(b"logins"),
        Item::directory(b"logins/smith"),
        Item::directory(b"logins/smith/tokens"),
        Item::file(b"logins/smith/tokens/chatgpt", b"synthetic marker; never read"),
    ]);
    let login = Path::new(LOGIN);
    let token = login.join("tokens/chatgpt");
    assert_eq!(fixture.tree.metadata(&token).expect("source marker").kind, Kind::File);
    for path in [
        login.to_path_buf(),
        login.join("tokens"),
        token,
        "/attempt/outside/tokens".into(),
        "/attempt/outside/tokens/chatgpt".into(),
    ] {
        fixture.tree.never_read(path);
    }
    let mut task = authored_task("paths");
    let private = fixture.context(&task).private;
    let app_path = private.workspace.join("app");
    for variant in [None, Some(&task.variant[0])] {
        let context = Context { variant, ..fixture.context(&task) };
        let rendered = fixture.render(&context).expect("path arguments");
        assert_eq!(&rendered.arguments[..2], ["-C", app_path.to_str().expect("path")]);
        assert!(rendered.arguments.iter().any(|argument| argument.starts_with("workspace.directories=")));
        let resolved = rendered.resolve();
        let mounts = &resolved.local.workspace.as_ref().expect("workspace").directories;
        assert_eq!(mounts.len(), if variant.is_some() { 1 } else { 2 });
        assert_eq!(mounts[0].name.as_ref(), b"app");
        assert!(mounts[0].writable);
        assert!(!mounts[0].git);
        assert_eq!(mounts[0].host.as_ref(), app_path.as_os_str().as_encoded_bytes());
        if variant.is_none() {
            assert_eq!(mounts[1].name.as_ref(), b"spec");
            assert!(!mounts[1].writable);
            assert!(!mounts[1].git);
            assert_eq!(mounts[1].host.as_ref(), private.workspace.join("spec").as_os_str().as_encoded_bytes());
        }
    }
    task.smith.remove("workspace.directories");
    let rendered = fixture.render(&fixture.context(&task)).expect("default mount arguments");
    assert!(!rendered.arguments.iter().any(|argument| argument.starts_with("workspace.directories=")));
    let resolved = rendered.resolve();
    let mounts = &resolved.local.workspace.as_ref().expect("default workspace").directories;
    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].host.as_ref(), app_path.as_os_str().as_encoded_bytes());
    mount_refusals(&fixture, &mut task);
    assert_eq!(fixture.tree.unexpected_reads(), []);
    assert_eq!(fixture.tree.writes(), []);
}

fn path_mounts() -> [Item; 2] {
    [Item::directory(b"attempt/workspace/app"), Item::directory(b"attempt/workspace/spec")]
}

fn mount_refusals(fixture: &Fixture, task: &mut smith_bench::Task) {
    let refusals = [
        "[]",
        "[{ name = 'app', path = '../outside', writable = true, git = false }]",
        "[{ name = 'app', path = '/tmp', writable = true, git = false }]",
        "[{ name = 'app', path = 'escaped', writable = true, git = false }]",
        "[{ name = 'app', path = 'absent', writable = true, git = false }]",
        "[{ name = 'app', path = 'spec', writable = true, git = false }]",
        "[{ name = 'app', path = 'app/input.txt', writable = true, git = false }]",
        "[{ name = '.', path = 'app', writable = true, git = false }]",
        "[{ name = 'a/b', path = 'app', writable = true, git = false }]",
        "[{ name = 'app', path = 'app', writable = true, git = false }, { name = 'app', path = 'spec', writable = false, git = false }]",
        "[{ name = 'app', path = 'app', writable = true, git = false }, { name = 'alias', path = './app', writable = false, git = false }]",
        "[{ name = 'root', path = '.', writable = true, git = false }, { name = 'app', path = 'app', writable = true, git = false }]",
        "[{ name = 'app', path = 'app', writable = true, git = false, extra = 1 }]",
        "[{ name = 'app', path = 'app', git = false }]",
    ];
    for row in refusals {
        let value: toml::Value = toml::from_str(&format!("directories = {row}")).expect("authored TOML");
        task.smith.insert("workspace.directories".into(), value["directories"].clone());
        let context = Context { home: Path::new("/attempt/outside"), ..fixture.context(task) };
        let refusal = fixture.render(&context).expect_err(row);
        assert_eq!(
            refusal.kind,
            smith_bench::FaultKind::Setup(smith_bench::isolate::SetupStep::Configuration),
            "{row}"
        );
        assert!(!fixture.tree.try_exists(&context.private.root.join("smith/tokens")).expect("token path"));
    }
}

#[test]
fn every_probe_tasks_overrides_pass_the_observed_public_startup_derivation() {
    let mut paths = std::fs::read_dir(probe_directory())
        .expect("probe tasks")
        .map(|entry| entry.expect("probe directory").path().join("task.toml"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    paths.sort();
    assert_ne!(paths.len(), 0);
    // Include the seed's sibling mounts for the paths probe; the other probes use the workspace root.
    let fixture = Fixture::new(&path_mounts());
    for path in paths {
        let task = read_probe(&path);
        let rendered =
            fixture.render(&fixture.context(&task)).unwrap_or_else(|why| panic!("{}: {why:?}", path.display()));
        let resolved = rendered.resolve();
        resolved.local.validate(&resolved.limits).expect("probe product policy");
    }
}

#[test]
fn tool_output_profiles_render_and_pass_the_observed_public_startup_derivation() {
    let fixture = Fixture::new(&[]);
    let task = authored_task("tool-output");
    assert_eq!(
        task.variant.iter().map(|variant| variant.name.as_str()).collect::<Vec<_>>(),
        ["test-output", "quote", "utf-8", "timeout", "background"]
    );
    for variant in &task.variant {
        let context = Context { variant: Some(variant), ..fixture.context(&task) };
        let resolved = fixture.render(&context).expect("tool-output settings").resolve();
        if variant.name == "timeout" {
            assert_eq!(resolved.agent.service.limits.domain.session.tools.tool_deadline, Duration::from_secs(1));
            assert_eq!(resolved.agent.service.limits.domain.session.tools.shell_timeout, Duration::from_secs(1));
            assert_eq!(resolved.agent.service.limits.machine.group_stop, Duration::from_secs(1));
        }
        assert_eq!(resolved.local.models.len(), 3);
        resolved.local.validate(&resolved.limits).expect("mounted product policy");
    }
}

#[test]
fn process_tree_profile_renders_and_passes_the_observed_public_startup_derivation() {
    let fixture = Fixture::new(&[]);
    let task = authored_task("process-tree");
    let resolved = fixture.render(&fixture.context(&task)).expect("profile settings").resolve();
    assert_eq!(resolved.agent.service.limits.domain.session.tools.tool_deadline, Duration::from_secs(1));
    assert_eq!(resolved.agent.service.limits.domain.session.tools.shell_timeout, Duration::from_secs(1));
    assert_eq!(resolved.agent.service.limits.machine.group_stop, Duration::from_secs(1));
    assert_eq!(resolved.local.models.len(), 3);
    resolved.local.validate(&resolved.limits).expect("product policy");
}

#[test]
fn slow_completion_profile_renders_and_passes_the_observed_public_startup_derivation() {
    let fixture = Fixture::new(&[]);
    let task = authored_task("slow-completion");
    let resolved = fixture.render(&fixture.context(&task)).expect("profile settings").resolve();
    assert_eq!(resolved.local.progress[0].head, Duration::from_millis(60000));
    assert_eq!(resolved.local.progress[0].idle, Duration::from_millis(300_000));
    assert_eq!(resolved.agent.declarations.endpoints[0].limits.reasoning_item, 65536);
    assert_eq!(
        resolved.agent.declarations.endpoints[0].limits.oversized_reasoning,
        smith_agent_service::profile::OversizedReasoning::Fail
    );
    assert_eq!(resolved.local.models.len(), 3);
    resolved.local.validate(&resolved.limits).expect("product policy");
}

#[test]
fn a_task_override_reaches_smith_as_a_product_setting() {
    let fixture = Fixture::new(&[]);
    let mut task = authored_task("first-request");
    task.smith.insert("models.\"{model}\".output".into(), toml::Value::Integer(8192));
    let rendered = fixture.render(&fixture.context(&task)).expect("overrides");
    assert!(rendered.arguments.contains(&"models.\"openai/gpt-6-luna\".output=8192".into()));
    assert_eq!(rendered.resolve().local.models[0].output, 8192);
}

#[test]
fn a_comparison_pin_keeps_every_preset_value_and_the_default_role() {
    let fixture = Fixture::new(&[]);
    let mut task = authored_task("first-request");
    task.smith.clear();
    task.budget.turns = None;
    task.budget.seconds = None;
    task.budget.spend = None;
    let rendered = fixture.render(&fixture.context(&task)).expect("product settings");
    let settings = rendered.settings();
    let preset_key = |key: &str| {
        key.starts_with("models.")
            || (key.starts_with("endpoints.") && key != "endpoints.openai.account")
            || key.starts_with("deployment.")
            || key == "instructions"
    };
    for (key, value) in &settings.values {
        if preset_key(key) {
            assert!(
                !matches!(
                    value.origin,
                    smith_local_shell::settings::Origin::File { layer: smith_local_shell::settings::Layer::User, .. }
                ),
                "{key}"
            );
        }
    }
    let recorded: serde_json::Value =
        serde_json::from_str(include_str!("../recorded/smith-product-check.json")).expect("recorded product check");
    assert_eq!(recorded["version"], 2);
    for (key, leaf) in recorded["settings"]["values"].as_object().expect("recorded leaves") {
        if preset_key(key) || leaf["origin"]["kind"] == "built_in" {
            assert_eq!(leaf["origin"]["kind"], "built_in", "{key}");
            assert_eq!(
                serde_json::to_value(&settings.values[key].origin).expect("setting origin"),
                leaf["origin"],
                "{key}"
            );
            assert_eq!(
                serde_json::to_value(&settings.values[key].value).expect("setting value"),
                leaf["value"],
                "{key}"
            );
        }
    }
    let resolved = rendered.resolve();
    assert!(!resolved.local.instructions.is_empty());
}

#[test]
fn a_task_without_a_budget_renders_the_products_defaults() {
    let fixture = Fixture::new(&[]);
    let mut task = authored_task("first-request");
    task.smith.remove("budget.reserve.seconds");
    task.budget.turns = None;
    task.budget.seconds = None;
    task.budget.spend = None;
    let rendered = fixture.render(&fixture.context(&task)).expect("product settings");
    let settings = rendered.settings();
    for (key, expected) in [("budget.turns", 512), ("budget.seconds", 3600)] {
        assert_eq!(settings.values[key].value.as_integer(), Some(expected));
        assert!(matches!(settings.values[key].origin, smith_local_shell::settings::Origin::BuiltIn));
    }
    assert!(!rendered.arguments.iter().any(|argument| argument.starts_with("budget.")));
}

#[test]
fn the_credentials_lead_reaches_the_native_policy_boundary() {
    let fixture = Fixture::new(&[]);
    let task = authored_task("credentials");
    let rendered = fixture.render(&fixture.context(&task)).expect("credentials flags");
    assert!(rendered.arguments.contains(&"deployment.refresh_lead=34560000".into()));
    let resolved = rendered.resolve();
    assert_eq!(resolved.settings.values["deployment.refresh_lead"].value.as_integer(), Some(34_560_000));
    assert_eq!(resolved.agent.profile.policy.refresh_lead, Duration::from_secs(34_560_000));
}

#[test]
fn the_current_source_arm_selects_the_chat_face() {
    use smith_bench::agent::smith::{events::Shape, read_faces_with, select_face};
    let tree = Tree::new(&[Item::file(b"faces.toml", include_bytes!("../../agents/smith/faces.toml"))]);
    let entries = read_faces_with(&tree, Path::new("/faces.toml")).expect("registry");
    let arm = smith_bench::arm::BuiltArm {
        version: 1,
        commit: "a".repeat(40),
        sha256: "b".repeat(64),
        toolchain: "supplied identity".into(),
        binaries: vec![],
    };
    for (statuses, shape, entry) in
        [(&[Some(0)][..], Shape::Chat, &entries[1]), (&[Some(1), Some(0)][..], Shape::Exec, &entries[0])]
    {
        let selected = select_face(&arm, &entries, statuses).expect("supported ancestor");
        assert_eq!(selected.shape, shape);
        assert_eq!(selected.commit, entry.commit);
        assert_eq!(selected.receiving, entry.receiving);
    }
    for status in [Some(2), Some(128), None] {
        let refusal = select_face(&arm, &entries, &[status, Some(0)]).expect_err("undetermined ancestry");
        assert_eq!(refusal.detail, "cannot establish Smith arm source ancestry");
    }
    let refusal = select_face(&arm, &entries, &[Some(1), Some(1)]).expect_err("no supported ancestor");
    assert_eq!(refusal.detail, format!("Smith arm {} predates the first supported product face", arm.commit));
}
