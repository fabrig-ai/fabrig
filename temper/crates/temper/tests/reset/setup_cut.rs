//! Owned setup started over after a crash (temper/docs/design/draft/playground-reset.md, 4, 6.4, 8).
use crate::support::{
    self, Ended,
    ctl::{self, Call, Step},
    forge_land, forges,
};
use skein_io::kernel::Exit;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
enum Cut {
    Fetch,
    Preparation,
    Running,
    FirstRepository,
}

#[test]
fn an_owned_set_up_killed_at_its_fetch_starts_over_from_the_cache() {
    starts_over(Cut::Fetch);
}

#[test]
fn an_owned_set_up_killed_at_its_preparation_starts_over_from_the_cache() {
    starts_over(Cut::Preparation);
}

#[test]
fn an_owned_set_up_killed_with_forgejo_running_starts_over_from_the_cache() {
    starts_over(Cut::Running);
}

#[test]
fn an_owned_set_up_killed_before_its_first_repository_starts_over_from_the_cache() {
    starts_over(Cut::FirstRepository);
}

fn starts_over(cut: Cut) {
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let scratch = support::Scratch::new("owned-setup-cut");
    let (paths, flags) = crate::first_run_directories(&scratch);
    let initialized = std::process::Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(&flags)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdin(std::process::Stdio::null())
        .current_dir(scratch.path())
        .output()
        .expect("fresh temper init");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    support::own_listener_files(&paths[0]);
    let (cached, sources) = forge_land::cache_forgejo_binaries(&paths[2]);
    let seen = support::boxed::run_ctl(
        scratch.path(),
        support::boxed::echo_steps(flags),
        scratch.path(),
        setup_cut_steps(cut, &paths[1]),
        support::boxed::BOX_DEADLINE,
        |_| (),
    );
    let stderr = String::from_utf8_lossy(&seen.stderr);
    assert_eq!(seen.checked, Some(()), "all story calls completed: {stderr}");
    assert_eq!(seen.exit, Some(Exit::Code(0)), "restarted box exits cleanly: {stderr}");
    assert_eq!(seen.left, Some(Vec::new()), "nothing outlived the restarted box: {stderr}");
    let [killed, restarted] = seen.runs.as_slice() else { panic!("two box starts: {stderr}") };
    assert_eq!(killed.exit, Some(Exit::Signal(9)), "the first box was killed");
    assert_eq!(restarted.exit, Some(Exit::Code(0)));
    assert_eq!(restarted.left, Some(Vec::new()));
    for (index, call) in seen.calls.iter().enumerate() {
        let expected = if index == 3 { 143 } else { 0 };
        assert_eq!(
            call.exit,
            Some(Exit::Code(expected)),
            "call {index}: stdout {}; stderr {}; box: {stderr}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
        assert!(call.stderr.is_empty(), "call {index}: {}", String::from_utf8_lossy(&call.stderr));
    }
    let cut_at = if matches!(cut, Cut::FirstRepository) { 5 } else { 4 };
    let row = only_forge(&seen.calls[cut_at], 1);
    eprintln!("{cut:?} cut row: {row}");
    assert_cut(cut, &row);
    let [_signed_again, unfinished, removing, absent, made, setup, connected, projects] = &seen.calls[cut_at + 1..]
    else {
        panic!("restart, remove, make and setup calls: {stderr}")
    };
    assert_unfinished(unfinished);
    assert_eq!(removing.stdout, b"removing\n");
    assert_empty(absent);
    assert_removed(&paths[1]);
    let replacement = forges::made(made);
    assert!(replacement > 1, "forge numbers are never reused");
    let row = only_forge(connected, replacement);
    assert_eq!(row["state"], "connected", "{row}");
    assert_eq!(row["first"]["state"], "made", "{row}");
    assert_eq!(row["first"]["org"], "work");
    assert_eq!(row["first"]["repository"], "hello");
    assert!(row["first"]["id"].as_u64().is_some_and(|id| id != 0), "{row}");
    let adoption = String::from_utf8_lossy(&setup.stdout);
    assert!(adoption.starts_with("setting up\n"), "{adoption}");
    assert!(adoption.contains(&format!("Adopted work/hello on forge {replacement} as owned.")), "{adoption}");
    let projects = ctl::json(projects);
    assert!(
        projects["projects"].as_array().expect("projects").iter().any(|project| project["name"]["bytes"] == "hello"),
        "{projects}"
    );
    let stderr = String::from_utf8_lossy(&restarted.stderr);
    assert!(
        !stderr.lines().any(|line| line.starts_with("fetch ") || line.contains("download")),
        "the restarted group downloads no binaries: {stderr}"
    );
    for step in ["forgejo", "runner"] {
        assert!(
            stderr.lines().any(|line| line.starts_with(&format!("{step} {replacement}: "))),
            "{step} ran: {stderr}"
        );
    }
    for (path, source) in cached.iter().zip(&sources) {
        forge_land::assert_cached_binary_unchanged(path, source);
        assert!(!path.with_extension("part").exists(), "no partial download");
    }
    eprintln!(
        "{cut:?}: unchanged cached binaries; no fetch/download after restart; forge 1 removed, forge {replacement} connected and first repository adopted; box exit 0, no remaining processes"
    );
    eprintln!("{cut:?} setup cut story: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    scratch.remove();
}

fn setup_cut_steps(cut: Cut, data: &Path) -> Vec<Step> {
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(forges::make_here()),
        Step::Then(Box::new(|calls| {
            assert_eq!(forges::made(&calls[1]), 1, "fresh init's first forge, also the stderr prefix");
            forge_list()
        })),
        Step::Start(Call::new(&["forge", "setup", "1"])),
    ];
    let gate: &'static [u8] = match cut {
        Cut::Fetch => b"\"state\":\"setting_up\"",
        Cut::Preparation => b"\"step\":\"prepare\"",
        Cut::Running | Cut::FirstRepository => b"\"step\":\"run\"",
    };
    steps.push(Step::Until(forge_list(), gate));
    match cut {
        Cut::Running => steps.push(Step::Stderr(b"forgejo 1: ")),
        Cut::FirstRepository => steps.push(Step::Until(forge_list(), b"\"state\":\"connecting\"")),
        Cut::Fetch | Cut::Preparation => {}
    }
    let removed_data = data.to_path_buf();
    steps.extend([
        Step::KillBox,
        Step::StartBox,
        Step::Run(forge_list()),
        Step::Then(Box::new(|calls| {
            assert_unfinished(calls.last().expect("forge read after restart"));
            forges::remove(1)
        })),
        Step::Until(forge_list(), b"\"forges\":[]"),
        Step::Then(Box::new(move |calls| {
            assert_empty(calls.last().expect("forge list after removal"));
            assert_eq!(calls[calls.len() - 2].stdout, b"removing\n");
            assert_removed(&removed_data);
            forges::make_here()
        })),
        Step::Then(Box::new(|calls| {
            let replacement = forges::made(calls.last().expect("new forge"));
            assert!(replacement > 1, "forge numbers are never reused");
            Call::new(&["forge", "setup", &replacement.to_string()])
        })),
        Step::Until(forge_list(), b"\"state\":\"connected\""),
        Step::Run(Call::new(&["me"]).json()),
    ]);
    steps
}

fn forge_list() -> Call {
    Call::new(&["forge", "list"]).json()
}

fn only_forge(call: &Ended, number: u16) -> serde_json::Value {
    let document = ctl::json(call);
    let [row] = document["forges"].as_array().expect("forge rows").as_slice() else {
        panic!("only the owned forge: {document}")
    };
    assert_eq!(row["number"], number);
    assert_eq!(row["mode"], "owned");
    row.clone()
}

fn assert_cut(cut: Cut, row: &serde_json::Value) {
    match cut {
        Cut::Fetch => {
            assert_eq!(row["state"], "setting_up");
            assert!(row["step"].is_null() || matches!(row["step"].as_str(), Some("fetch" | "prepare")), "{row}");
        }
        Cut::Preparation | Cut::Running => {
            assert_eq!(row["state"], "setting_up");
            assert_eq!(row["step"], if matches!(cut, Cut::Preparation) { "prepare" } else { "run" });
        }
        Cut::FirstRepository => {
            assert_eq!(row["state"], "connecting");
            assert!(matches!(row["first"]["state"].as_str(), Some("making" | "made")), "{row}");
        }
    }
}

fn assert_unfinished(call: &Ended) {
    let row = only_forge(call, 1);
    assert!(row["state"] != "connected" || row["first"]["state"] != "made", "kept but unfinished: {row}");
    eprintln!("unfinished after restart: {row}");
}

fn assert_empty(call: &Ended) {
    let document = ctl::json(call);
    assert!(document["forges"].as_array().expect("forge rows").is_empty(), "{document}");
}

fn assert_removed(data: &Path) {
    for path in ["box/groups/1", "box/started/1", "forge-1"] {
        assert!(!data.join(path).exists(), "remove erased {path}");
    }
}
