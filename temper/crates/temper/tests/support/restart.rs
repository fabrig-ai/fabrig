//! Executable restart boundaries on minimal deployments.
//! temper/docs/design/domain/testing.md, 2.6 and 6.1.
use super::{
    Ended, Scratch, boxed,
    ctl::{self, Call, Step},
    forge_land::{self, cache_forgejo_binaries, deployment_flags, forgejo_setup_steps},
};
use boxed::BOX_DEADLINE;
use skein_io::kernel::Exit;

fn drained(exit: Option<Exit>, left: Option<&[u32]>, stderr: &[u8]) {
    assert_eq!(exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(stderr));
    assert_eq!(left, Some([].as_slice()), "no process outlived a box: {}", String::from_utf8_lossy(stderr));
}

fn answered(calls: &[Ended], count: usize) {
    assert_eq!(calls.len(), count, "every ordinary ctl call completed");
    for call in calls {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "stdout: {}; stderr: {}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
}

fn reads() -> Vec<Step> {
    vec![
        Step::Run(Call::new(&["settings", "show", "--json"])),
        Step::Run(Call::new(&["account", "list", "--json"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
    ]
}

/// Both owned services use the pinned cache while the first deployment restarts.
pub fn shared_cache_restart(case: &str) {
    let scratch = Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    forge_land::initialize(&paths);
    super::own_listener_files(&paths[0]);
    let _cached = cache_forgejo_binaries(&paths[2]);
    let flags = boxed::echo_steps(deployment_flags(&paths));
    let first = boxed::run_ctl(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        forgejo_setup_steps("first"),
        BOX_DEADLINE,
        |_| (),
    );
    drained(first.exit, first.left.as_deref(), &first.stderr);
    answered(&first.calls, 7);
    assert!(first.checked.is_some(), "initial owned deployment completed setup");

    let other = scratch.path().join("other");
    std::fs::create_dir(&other).expect("second deployment");
    let mut other_paths = ["config", "data", "cache"].map(|name| other.join(name));
    forge_land::initialize(&other_paths);
    super::own_listener_files(&other_paths[0]);
    other_paths[2].clone_from(&paths[2]);
    let other = boxed::run_ctl(
        &other,
        boxed::echo_steps(deployment_flags(&other_paths)),
        &other,
        forgejo_setup_steps("first"),
        BOX_DEADLINE,
        |_| {
            let mut steps = forgejo_setup_steps("again");
            steps.extend(reads());
            boxed::run_ctl(scratch.path(), flags, scratch.path(), steps, BOX_DEADLINE, |_| ())
        },
    );
    drained(other.exit, other.left.as_deref(), &other.stderr);
    answered(&other.calls, 7);
    let restarted = other.checked.expect("the second owned service stayed live throughout the first's restart");
    drained(restarted.exit, restarted.left.as_deref(), &restarted.stderr);
    answered(&restarted.calls, 7);
    assert!(restarted.checked.is_some(), "all restart reads completed");
    scratch.remove();
}

/// The one-shot listener abort is supervised before ordinary ctl reads succeed.
pub fn cut_at_listening_restart(case: &str) {
    let scratch = Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    let issuer = forge_land::fake_deployment(&paths);
    let flags = boxed::echo_steps(deployment_flags(&paths));
    let mut steps = vec![Step::Run(Call::sign_in())];
    steps.extend(reads());
    let first = boxed::run_ctl_with_account(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        steps,
        BOX_DEADLINE,
        || (),
        forge_land::fake_face(issuer, temper_real_loop_world::llm::cued(Box::new([]))),
    );
    drained(first.exit, first.left.as_deref(), &first.stderr);
    answered(&first.calls, 4);
    assert!(first.checked.is_some(), "initial empty deployment completed its reads");
    let marker = scratch.path().join("start-cut");
    let restarted = boxed::run_ctl_with_account_probe(
        scratch.path(),
        flags,
        scratch.path(),
        reads(),
        BOX_DEADLINE,
        || (),
        boxed::AccountProbe {
            face: forge_land::fake_face(
                super::sign_in::restart_issuer(&paths[0]),
                temper_real_loop_world::llm::cued(Box::new([])),
            ),
            cut_after_listening: Some(marker.clone()),
        },
    );
    drained(restarted.exit, restarted.left.as_deref(), &restarted.stderr);
    answered(&restarted.calls, 3);
    assert!(restarted.checked.is_some(), "all restart reads completed");
    assert!(marker.exists(), "the intermediate start was cut exactly at listening");
    let stderr = String::from_utf8_lossy(&restarted.stderr);
    assert!(stderr.contains("serve killed by signal 6"), "box restarted the cut serve: {stderr}");
    assert!(!stderr.contains("panicked at"), "only the injected abort occurred: {stderr}");
    scratch.remove();
}

fn fake_flags(scratch: &Scratch) -> Vec<String> {
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    forge_land::initialize(&paths);
    super::own_listener_files(&paths[0]);
    let bundle = scratch.path().join("bundle");
    std::fs::create_dir_all(bundle.join("0000000000000001")).expect("fixture bundle");
    super::write_private(
        &bundle.join("index.html"),
        b"<!doctype html><html><head></head><body>restart fixture</body></html>",
    );
    super::write_private(&bundle.join("0000000000000001/main.js"), b"// process fixture\n");
    let mut flags = deployment_flags(&paths);
    flags.extend(["--fake-backends".into(), "--bundle".into(), bundle.display().to_string()]);
    flags
}

fn review_fixture(config: &std::path::Path) {
    for file in ["temper.json", "agent.json"] {
        let path = config.join(file);
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("effective configuration")).expect("fixture JSON");
        if file == "temper.json" {
            cheap_models(&mut value["engine"]["llm"]["models"]);
            let llm = &mut value["application"]["demo"]["llm"];
            let chat = llm["asked"]
                .as_array_mut()
                .expect("chat scripts")
                .iter_mut()
                .find(|script| script["cue"] == "Add CSV export to widget")
                .expect("delegating chat");
            chat["turns"][0]["lines"]
                .as_array_mut()
                .expect("delegation lines")
                .iter_mut()
                .find(|line| line["call"] == "delegate")
                .expect("stock delegation")["arguments"]["batch"][0]["authority"]["budget"]["spend"] = 10.into();
            let review = llm["cued"]
                .as_array_mut()
                .expect("agent scripts")
                .iter_mut()
                .find(|script| script["cue"] == "Review this change at the named head")
                .expect("review script");
            review["turns"] = serde_json::json!(vec![
                serde_json::json!({
                    "lines":[{"call":"read","arguments":{"path":"src/main.rs"}}],
                    "finish":"tool_calls","tokens":20
                });
                16
            ]);
        } else {
            for endpoint in value["endpoints"].as_array_mut().expect("agent endpoints") {
                cheap_models(&mut endpoint["models"]);
            }
        }
        super::write_private(&path, &serde_json::to_vec(&value).expect("effective fixture JSON"));
    }
}

fn cheap_models(models: &mut serde_json::Value) {
    for model in models.as_array_mut().expect("fixture models") {
        model["prices"] = serde_json::json!({"input":1,"cached":1,"output":1,"unit":1_000_000});
    }
}

/// The two reviewer reads prove the hold, independently of unrelated change panels.
pub fn assert_held_reviewers(calls: &[Ended], before_call: bool) {
    for task in [4, 8] {
        let reviewer = calls
            .iter()
            .rev()
            .filter_map(|call| serde_json::from_slice::<serde_json::Value>(&call.stdout).ok())
            .find(|row| row["chip"]["task"] == task)
            .expect("review task read");
        assert_eq!(reviewer["chip"]["phase"], "held", "{reviewer}");
        let spent = reviewer["budget"]["numbers"]["spent"].as_u64().expect("review spend");
        if before_call {
            assert_eq!(spent, 0, "review never called its model: {reviewer}");
            assert_eq!(
                reviewer["chip"]["details"]["owner"]["details"]["run_since"],
                serde_json::Value::Null,
                "review never began a run: {reviewer}"
            );
        } else {
            assert_eq!(reviewer["hold"]["owner"]["reason"], "budget", "budget hold: {reviewer}");
            assert!(spent > 0, "review ran: {reviewer}");
        }
    }
}

fn held_review_steps(scratch: &std::path::Path, before_call: bool) -> Vec<Step> {
    let parking = scratch.join("review-parking.json");
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["playground", "start", "--key", "review-playground"])),
        Step::Run(Call::new(&["settings", "show", "--json"])),
        Step::Run(Call::new(&["settings", "show", "charter", "1", "--json"])),
        Step::Run(Call::new(&["settings", "show", "template", "1", "--json"])),
        Step::Then(Box::new(move |calls| {
            let mut settings = [ctl::json(&calls[3]), ctl::json(&calls[4])];
            settings[0]["value"]["waiting"] = 1_000_000.into();
            settings[1]["value"]["budget"] = serde_json::json!({"as_allowed":{"cap":64}});
            let items =
                settings.map(|setting| serde_json::json!({"setting":setting["setting"],"set":setting["value"]}));
            super::write_private(
                &parking,
                &serde_json::to_vec(&serde_json::json!({"items":items})).expect("review settings JSON"),
            );
            ctl::settings_change(&calls[2], &parking, "review-parking")
        })),
    ];
    if before_call {
        let hosts = scratch.join("review-hosts.json");
        steps.extend([
            Step::Run(Call::new(&["settings", "show", "--json"])),
            Step::Run(Call::new(&["settings", "show", "charter", "4", "--json"])),
            Step::Then(Box::new(move |calls| {
                let last = calls.len() - 1;
                ctl::setting_change_file(&hosts, &calls[last], |value| value["hosts"] = "engine".into());
                ctl::settings_change(&calls[last - 1], &hosts, "review-hosts")
            })),
        ]);
    }
    for (chat, change, review, key) in [("1", "2", "4", "first-review"), ("5", "6", "8", "second-review")] {
        steps.extend([
            Step::Run(Call::new(&[
                "chat",
                "start",
                "--project",
                "1",
                "--model",
                "demo",
                "--key",
                key,
                "Add CSV export to widget",
                "--json",
            ])),
            Step::Run(Call::new(&["task", "show", chat, "--until", "replied", "--timeout", "120s", "--json"])),
            Step::Run(Call::new(&["change", "show", change, "--until", "ci:passed", "--timeout", "120s", "--json"])),
            Step::TaskExists {
                project: 1,
                parent: change.parse().expect("change task"),
                task: review.parse().expect("review task"),
            },
        ]);
        if before_call {
            steps.push(Step::Until(Call::new(&["task", "show", review, "--json"]), b"\"phase\":\"due\""));
            steps.push(Step::Run(Call::new(&["task", "stop", review])));
        }
        steps.extend([
            Step::Until(Call::new(&["task", "show", review, "--json"]), b"\"phase\":\"held\""),
            Step::Run(Call::new(&["task", "show", review, "--json"])),
        ]);
    }
    steps
}

/// Two held reviewers on the playground survive a real box stop and restart.
pub fn held_reviews_restart(case: &str, before_call: bool) {
    let scratch = Scratch::new(case);
    let flags = fake_flags(&scratch);
    let prepared = boxed::run_ctl(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        vec![Step::Run(Call::sign_in())],
        BOX_DEADLINE,
        |_| (),
    );
    drained(prepared.exit, prepared.left.as_deref(), &prepared.stderr);
    answered(&prepared.calls, 1);
    assert!(prepared.checked.is_some(), "empty fake-backend box prepared");
    review_fixture(&scratch.path().join("data/box/fake-backends/config"));
    let steps = held_review_steps(scratch.path(), before_call);
    let first = boxed::run_ctl(scratch.path(), flags.clone(), scratch.path(), steps, BOX_DEADLINE, |_| ());
    drained(first.exit, first.left.as_deref(), &first.stderr);
    answered(&first.calls, if before_call { 23 } else { 16 });
    assert!(first.checked.is_some(), "both reviewers reached their holds");
    assert_held_reviewers(&first.calls, before_call);
    let mut steps = reads();
    steps.extend([
        Step::Run(Call::new(&["account", "list", "--until", "usable", "--timeout", "10s"])),
        Step::Run(Call::new(&["task", "show", "4", "--json"])),
        Step::Run(Call::new(&["task", "show", "8", "--json"])),
    ]);
    let restarted = boxed::run_ctl(scratch.path(), flags, scratch.path(), steps, BOX_DEADLINE, |_| ());
    drained(restarted.exit, restarted.left.as_deref(), &restarted.stderr);
    answered(&restarted.calls, 6);
    assert!(restarted.checked.is_some(), "settings, account, forge and reviewer reads completed");
    assert_held_reviewers(&restarted.calls, before_call);
    scratch.remove();
}

/// A stopped fake-forge change leaves a real watch alive across an engine restart.
pub fn change_wait_restart(case: &str) {
    let scratch = Scratch::new(case);
    let flags = fake_flags(&scratch);
    let hosts = scratch.path().join("producer-hosts.json");
    let steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["playground", "start", "--key", "restart-playground"])),
        Step::Run(Call::new(&["settings", "show", "--json"])),
        Step::Run(Call::new(&["settings", "show", "charter", "2", "--json"])),
        Step::Then(Box::new(move |calls| {
            // No producer can run on this standalone worker before the change is stopped.
            ctl::setting_change_file(&hosts, &calls[3], |value| value["hosts"] = "engine".into());
            ctl::settings_change(&calls[2], &hosts, "restart-producer-hosts")
        })),
        Step::Run(Call::new(&[
            "chat",
            "start",
            "--project",
            "1",
            "--model",
            "demo",
            "--key",
            "restart-change",
            "Add CSV export to widget",
            "--json",
        ])),
        Step::TaskExists { project: 1, parent: 1, task: 2 },
        Step::Run(Call::new(&["task", "stop", "2"])),
        Step::Run(Call::new(&["task", "show", "2", "--until", "held", "--timeout", "10s", "--json"])),
        Step::Start(Call::new(&["change", "show", "2", "--until", "ci:failed", "--timeout", "12s"])),
        Step::RestartEngine,
        Step::Join,
    ];
    let outcome = boxed::run_ctl(scratch.path(), flags, scratch.path(), steps, BOX_DEADLINE, |_| ());
    drained(outcome.exit, outcome.left.as_deref(), &outcome.stderr);
    assert!(outcome.checked.is_some(), "watch joined after the clean engine restart");
    let (wait, setup) = outcome.calls.split_last().expect("started change watch");
    answered(setup, 8);
    let held = ctl::json(setup.last().expect("held change task"));
    let held = &held["task"];
    assert_eq!(held["chip"]["phase"], "held", "{held}");
    let task = held["chip"]["task"].as_u64().expect("held change identity");
    assert_eq!(task, 2, "the chat's delegated change is stopped");
    assert_eq!(wait.exit, Some(Exit::Code(3)), "{}", String::from_utf8_lossy(&wait.stderr));
    assert!(wait.stderr.is_empty());
    let panel = String::from_utf8_lossy(&wait.stdout);
    assert!(panel.contains(&format!("held: task {task}")) && panel.contains("not met: timeout"), "{panel}");
    assert!(String::from_utf8_lossy(&outcome.stderr).contains("restarting"), "engine restarted");
    scratch.remove();
}
