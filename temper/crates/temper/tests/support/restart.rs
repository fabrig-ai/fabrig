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

/// A stopped fake-forge change leaves a real watch alive across an engine restart.
pub fn change_wait_restart(case: &str) {
    let scratch = Scratch::new(case);
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
