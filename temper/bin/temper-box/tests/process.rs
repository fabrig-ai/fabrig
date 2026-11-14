//! The shipped binary, on a scratch deployment, drained on a signal.
#[path = "suite/box_binary.rs"]
mod box_binary;
#[path = "suite/box_kinds.rs"]
mod box_kinds;
#[path = "suite/box_lifetime.rs"]
mod box_lifetime;
use temper_box_kit::box_link;
#[path = "suite/client_binary.rs"]
mod client_binary;
#[path = "fake_backends/mod.rs"]
mod fake_backends;
#[path = "suite/startover.rs"]
mod startover;
use temper_box_kit as support;
#[path = "suite/tui_box.rs"]
mod tui_box;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;
use support::forge_land::{
    assert_cached_binary_unchanged, assert_forgejo_setup, assert_plan_box, cache_forgejo_binaries, deployment_flags,
    forgejo_setup_steps, plan_change_steps,
};

use support::boxed::BOX_DEADLINE;

#[test]
fn an_adopt_cut_by_a_signal_is_made_once() {
    use support::ctl::{Call, PROJECT, Step, Story, json};
    let scratch = support::Scratch::new("ctl_cut");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let adopt = &[
        "repository",
        "adopt",
        "--project",
        PROJECT,
        "--owner",
        "acme",
        "--name",
        "widget",
        "--landing",
        "main",
        "--key",
        "widget",
        "--json",
    ];
    let seen = support::ctl::run(Story::on(
        scratch.path(),
        &forgejo,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Until(Call::new(&["forge", "list", "--json"]), b"\"state\":\"connected\""),
            Step::Pause,
            Step::Start(Call::new(adopt)),
            Step::Until(Call::new(&["pending"]), b"adopt"),
            Step::Cut,
            Step::Run(Call::new(&["pending", "--json"])),
            Step::Resume,
            Step::Run(Call::new(adopt)),
            Step::Run(Call::new(&["pending"])),
        ],
    ));
    let [_, me, connected, cut, _, listed, again, empty] = seen.calls.as_slice() else { panic!("eight ctl ends") };
    assert_eq!(json(connected)["forges"][0]["state"], "connected");
    assert_eq!((cut.exit, cut.stdout.as_slice()), (Some(Exit::Code(143)), b"".as_slice()), "128 + SIGTERM");
    let (party, listed) = (json(me)["party"]["number"].as_u64().expect("signed in"), json(listed));
    let first_sent = listed["pending"][0]["first_sent"].clone();
    assert!(first_sent.is_u64() || first_sent.is_null(), "{listed}");
    let row = serde_json::json!({
        "operation": "adopt", "party": party, "first_sent": first_sent,
        "unanswered": "it clears when the engine answers it, or past its retention; pending lists it"
    });
    assert_eq!(listed, serde_json::json!({"pending": [row], "unread": 0}));
    assert_eq!(again.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&again.stderr));
    let adopted = json(again);
    assert_eq!(adopted["adopted"]["owner"], "acme");
    assert_eq!(adopted["adopted"]["name"], "widget");
    assert!(again.stderr.is_empty(), "the resumed call prints its first answer once, on stdout");
    assert_eq!((empty.exit, empty.stdout.as_slice()), (Some(Exit::Code(0)), b"".as_slice()));
    forgejo.stop();
    scratch.remove();
}

#[test]
fn temper_open_adopts_a_repository_by_commands() {
    let scratch = support::Scratch::new("temper_open_adopt");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let mut serve = support::serve(scratch.path());
    let link = support::terminal::link(&mut serve, support::WITHIN);
    let open = support::terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut typist = support::terminal::Typist::new(open, serve, link);
    typist.wait_for(b"== Inbox == Live", support::WITHIN);
    typist.wait_for(b"All goals (pressed)\n", support::WITHIN);
    typist.type_on(b"Deployment", b"");
    typist.type_on(b"Forges", b"");
    typist.wait_for(b"Forge 1 \xc2\xb7 Forgejo \xc2\xb7 Connected as temper", support::WITHIN);
    typist.type_on(b"How work runs", b"");
    typist.wait_for(b"== How work runs in ", support::WITHIN);
    typist.type_on(b"Repositories", b"");
    typist.type_on(b"Add repositories", b"");
    for (control, words, printed) in [
        (b"Owner: (empty)".as_slice(), b"acme".as_slice(), b"Owner: acme".as_slice()),
        (b"Name: (empty)", b"widget", b"Name: widget"),
        (b"Landing branch: (empty)", b"main", b"Landing branch: main"),
        (
            b"Checks: gate numbers, separated by spaces or commas: (empty)",
            b"1",
            b"Checks: gate numbers, separated by spaces or commas: 1",
        ),
    ] {
        typist.type_on(control, words);
        typist.wait_for(printed, support::WITHIN);
    }
    typist.type_on(b"[x] It runs CI", b"");
    typist.wait_for(b"[ ] It runs CI", support::WITHIN);
    typist.type_on(b"Adopt acme/widget", b"");
    typist.wait_for(b"## Repositories \xc2\xb7 1", support::WITHIN);
    typist.type_on(b"How work runs", b"");
    typist.wait_for(b"== How work runs in ", support::WITHIN);
    typist.type_on(b"Repositories", b"");
    typist.wait_for(b"## Repositories \xc2\xb7 1", support::WITHIN);
    typist.wait_for(b"acme/widget", support::WITHIN);
    typist.type_line(b"q");
    assert_eq!(typist.exit(support::WITHIN), 0, "stderr: {}", typist.stderr());
    forgejo.stop();
    scratch.remove();
}

#[test]
fn a_push_is_delivered_signed_with_the_secret_reconcile_drew() {
    let scratch = support::Scratch::new("hook_delivery");
    let (forgejo, listener) = support::forgejo::Throwaway::start_receiving(&scratch);
    assert_eq!(
        std::net::TcpListener::bind(listener.local_addr().expect("delivery address"))
            .expect_err("receiver held")
            .kind(),
        std::io::ErrorKind::AddrInUse
    );
    forgejo.push("hook.txt");
    let delivery = loop {
        let delivery = support::webhooks::receive(&listener, support::WITHIN);
        if skein_lib::bytes::find(&delivery.body, b"Add hook.txt").is_some() {
            break delivery;
        }
    };
    assert_eq!(delivery.event, "push", "Forgejo names the event");
    assert!(skein_lib::bytes::find(&delivery.body, b"refs/heads/main").is_some(), "a push to main");
    assert_eq!(delivery.signature, support::webhooks::signature(&forgejo.secret(), &delivery.body));
    forgejo.stop();
    scratch.remove();
}

#[test]
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    landing("landed", true);
}

#[test]
fn a_turn_landing_is_found_after_a_runner_cancellation_and_restarts() {
    turn_landing_restarts("turn-retry-restarts", true);
}

/// temper/docs/design/testing/process-tier.md: a queue that lands green.
#[test]
fn a_batch_of_three_lands_by_one_fast_forward() {
    batch_landing("batch-three", false);
}

/// temper/docs/design/testing/process-tier.md: a red narrowed.
#[test]
fn a_red_batch_is_narrowed_before_the_remaining_change_lands() {
    batch_landing("batch-red-narrowed", true);
}

fn batch_landing(case: &str, narrowed: bool) {
    use support::{batch_landing, landing_by_turn, turn_restart};
    let scratch = support::Scratch::new(case);
    let barrier = support::forgejo::JobBarrier::new(scratch.path());
    let forge = turn_restart::start(&scratch);
    let workflow = barrier.gate_workflow(landing_by_turn::WORKFLOW);
    forge.add_file(".forgejo/workflows/gate.yaml", workflow.as_bytes());
    support::configuration(scratch.path(), &forge.forge());
    batch_landing::configure(scratch.path(), forge.repository());
    if narrowed {
        batch_landing::run_narrowed(scratch.path(), &forge, &barrier);
    } else {
        batch_landing::run(scratch.path(), &forge, &barrier);
    }
    forge.stop();
    scratch.remove();
}

fn turn_landing_restarts(case: &str, retry: bool) {
    use support::{landing_by_turn, turn_restart};
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let scratch = support::Scratch::new(case);
    let barrier = support::forgejo::JobBarrier::new(scratch.path());
    let forge = turn_restart::start(&scratch);
    let workflow = barrier.gate_workflow(landing_by_turn::WORKFLOW);
    forge.add_file(".forgejo/workflows/gate.yaml", workflow.as_bytes());
    support::configuration(scratch.path(), &forge.forge());
    turn_restart::configure(scratch.path(), forge.repository());
    let started = clock.now().now;
    let (proof, _) = turn_restart::run(scratch.path(), &forge, &barrier, None);
    eprintln!("turn restart first run: {}", landing_by_turn::elapsed(&clock, started));
    let proof = proof.expect("stopped inside the gate job");
    let panel = &proof.reads.landing;
    eprintln!("turn restart mid-proof: {}", serde_json::to_string(&proof.reads.landing).expect("landing JSON"));
    assert_eq!(panel["phase"], "proving");
    let member = &panel["members"][0];
    let candidate = member["candidate"].as_str().expect("candidate");
    let commit = &candidate[..40];
    assert_ne!(proof.original, commit, "the merge train combined the change");
    assert_ne!(proof.moved, commit, "the candidate has not landed yet");
    assert_eq!(member["base"].as_str().map(|base| &base[..40]), Some(proof.moved.as_str()));
    assert!(member["gates"][0]["proof"].get("dispatched").is_some());
    assert_eq!(forge.merged_pull(), None);
    let retried = if retry {
        forge.cancel_run(proof.run);
        let started = clock.now().now;
        let retried = turn_restart::retry(scratch.path(), &forge, &barrier, &proof);
        eprintln!("turn restart retry run: {}", landing_by_turn::elapsed(&clock, started));
        assert_eq!(retried.reads.landing["members"][0]["gates"][0]["key"], member["gates"][0]["key"]);
        assert_eq!(retried.reads.landing["members"][0]["candidate"], candidate);
        barrier.release(proof.run);
        Some(retried)
    } else {
        None
    };
    let proof = retried.as_ref().unwrap_or(&proof);
    let started = clock.now().now;
    barrier.release(proof.run);
    let (_, landed) = turn_restart::run(scratch.path(), &forge, &barrier, Some(proof));
    eprintln!("turn restart second run: {}", landing_by_turn::elapsed(&clock, started));
    let landed = landed.expect("landed reads before stopping");
    assert_eq!(landed.landing["phase"], "landed");
    assert_eq!(landed.change["step"], "landed");
    assert_eq!(landed.change["repairs"]["used"], 0, "runner faults do not repair the change");
    assert_eq!(landed.landing["members"][0]["candidate"], candidate);
    let gate = &landed.landing["members"][0]["gates"][0];
    let receipt = &gate["proof"]["passed"];
    assert_eq!(receipt["run"], proof.run, "the restart found the dispatched run");
    let merged = forge.merged_pull().expect("merged pull");
    let main = forge.branch_head(b"main").expect("main tip");
    assert_eq!(main.as_ref(), commit.as_bytes());
    assert!(forge.main_readme().ends_with(support::EDIT), "the produced edit reached main");
    let pull = forge.repository_observation("acme", "widget", &format!("pulls/{merged}"));
    assert_eq!(pull["head"]["sha"], commit);
    assert_eq!(pull["merge_commit_sha"], commit);
    let parents = forge.repository_observation("acme", "widget", &format!("git/commits/{commit}"));
    assert_eq!(parents["parents"].as_array().expect("parents").len(), 2);
    assert_eq!(parents["parents"][0]["sha"], proof.moved);
    assert_eq!(parents["parents"][1]["sha"], proof.original);
    let runs = forge.repository_observation("acme", "widget", &format!("actions/runs?head_sha={commit}&limit=100"));
    let runs: Vec<_> = runs["workflow_runs"]
        .as_array()
        .expect("workflow runs")
        .iter()
        .filter(|run| run["workflow_id"] == "gate.yaml")
        .collect();
    assert_eq!(runs.len(), if retry { 2 } else { 1 }, "only the expected dispatches for the key");
    let passed = runs.iter().find(|run| run["id"] == proof.run).expect("receipted run");
    for run in &runs {
        assert_eq!(run["commit_sha"], commit);
        assert_eq!(run["trigger_event"], "workflow_dispatch");
        assert_eq!(run["trigger_user"]["login"], "temper");
    }
    assert_eq!(passed["status"], "success");
    let statuses = forge.repository_observation("acme", "widget", &format!("commits/{commit}/status"));
    let status = statuses["statuses"]
        .as_array()
        .expect("statuses")
        .iter()
        .find(|status| status["context"] == "temper/gate/gate")
        .expect("gate status");
    assert_eq!(status["status"], "success");
    let key = gate["key"].as_str().expect("receipt key");
    assert_eq!(status["description"], format!("key {} passed", &key[..16]));
    assert_eq!(status["target_url"], receipt["address"]);
    eprintln!("turn restart landed reads: {landed:?}");
    forge.stop();
    scratch.remove();
    eprintln!("turn restart story wall: {}", landing_by_turn::elapsed(&clock, began));
}

/// The landing story; the hinted case raises four limits and pushes the slow job marker.
fn landing(case: &str, raised: bool) {
    let scratch = support::Scratch::new(case);
    let forgejo = if raised {
        support::forgejo::Throwaway::start_with_job_sleep(&scratch, 5)
    } else {
        support::forgejo::Throwaway::start(&scratch)
    };
    if raised {
        forgejo.push("slow");
    }
    support::configuration(scratch.path(), &forgejo.forge());
    if raised {
        support::raise_paces(scratch.path());
    }
    let peer = support::fake_llm(support::landing_scripts(support::ANSWER, &forgejo));
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let first = support::run_until_landed(scratch.path(), support::person::adopt_and_change(&forgejo), peer, &forgejo);
    assert_eq!((first.signed_in, first.finished), (1, true), "{}", String::from_utf8_lossy(&first.stderr));
    eprintln!("stage job: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    let merged = forgejo.merged_pull().expect("the change's pull request is merged");
    assert_eq!(forgejo.job_state(merged), "success");
    assert!(forgejo.main_readme().ends_with(support::EDIT), "main holds the agent's edit");
    let second = support::run(scratch.path(), support::person::found(), None, Duration::from_secs(120));
    assert!(second.finished, "the work is found after the restart");
    assert_eq!(forgejo.merged_pull(), Some(merged), "the forge's state is unchanged by the restart");
    forgejo.stop();
    scratch.remove();
}

#[test]
fn forge_prepare_makes_an_owned_forge_then_remakes_nothing() {
    let scratch = support::Scratch::new("forge_prepare_real");
    let _port = support::owned::prepared_processes(scratch.path());
    let before: Vec<_> = support::owned::files(scratch.path())
        .iter()
        .map(|path| {
            assert!(support::private(path), "{} is private", path.display());
            fs_read(path)
        })
        .collect();
    let secret = fs_read(&scratch.path().join("forge-1/runner/secret"));
    assert_eq!(secret.len(), 40);
    assert!(secret.iter().all(u8::is_ascii_hexdigit));
    let second = support::owned::prepare(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "{}", String::from_utf8_lossy(&second.stderr));
    let after: Vec<_> = support::owned::files(scratch.path()).iter().map(|path| fs_read(path)).collect();
    assert_eq!(after, before);
    let ini = scratch.path().join("forge-1/forgejo/app.ini");
    let settings = fs_read(&ini);
    let admin_token = scratch.path().join("forge-1/admin_token");
    std::fs::remove_file(&admin_token).expect("removed token");
    let third = support::owned::prepare(scratch.path(), 1);
    assert_eq!(third.status.code(), Some(0), "{}", String::from_utf8_lossy(&third.stderr));
    let token = fs_read(&admin_token);
    let line = token.split(|byte| *byte == b'\n').next().expect("token line");
    assert_eq!(line.len(), 40);
    assert!(line.iter().all(u8::is_ascii_hexdigit));
    assert_eq!(fs_read(&ini), settings);
    scratch.remove();
}
fn fs_read(path: &std::path::Path) -> Vec<u8> {
    std::fs::read(path).expect("prepared file")
}

#[test]
fn a_prepared_forge_runs_forgejo_and_its_runner_once_each() {
    use support::owned;
    let scratch = support::Scratch::new("forge_run_real");
    let port = owned::prepared_processes(scratch.path());
    let forgejo = owned::Running::start(scratch.path(), "forgejo");
    let version = owned::await_answer(port, "/api/v1/version", None);
    assert_eq!(
        version["version"].as_str().expect("version").split('+').next(),
        Some(skein_forge::forgejo::pin::VERSION)
    );
    let token = std::fs::read_to_string(scratch.path().join("forge-1/admin_token")).expect("admin token");
    let admin = owned::await_answer(port, "/api/v1/user", Some(token.trim()));
    assert_eq!(admin["login"], "temper-admin");
    assert_eq!(admin["is_admin"], true);
    let mut runner = owned::Running::start(scratch.path(), "runner");
    runner.declared();
    let refused = owned::command(scratch.path(), "forgejo").output().expect("second run");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge run: forge 1: forgejo is running or being prepared already\n");
    runner.stop();
    forgejo.stop();
    assert!(owned::answer(port, "/api/v1/version", None).is_err());
    scratch.remove();
}

#[test]
fn an_older_recorded_forgejo_is_dumped_then_upgraded() {
    use std::fs;
    use support::owned;
    use temper_forge_setup::{place::File, state};
    let scratch = support::Scratch::new("forge_upgrade_real");
    let _port = owned::prepared_processes(scratch.path());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let partial = owned::file(scratch.path(), 1, File::DumpPartial);
    fs::write(&partial, b"interrupted dump").expect("partial");
    let upgraded = owned::prepare(scratch.path(), 1);
    assert!(upgraded.status.success(), "{}", String::from_utf8_lossy(&upgraded.stderr));
    let (backup, before, meta) = owned::dump(scratch.path());
    assert!(!partial.exists());
    let record = state::decode(&fs::read(owned::file(scratch.path(), 1, File::State)).expect("upgrade fixture"))
        .expect("upgrade fixture");
    assert_eq!(&*record.forgejo, skein_forge::forgejo::pin::VERSION.as_bytes());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let repeated = owned::prepare(scratch.path(), 1);
    assert!(repeated.status.success(), "{}", String::from_utf8_lossy(&repeated.stderr));
    assert_eq!(fs::read(&backup).expect("backup kept"), before);
    assert_eq!(
        fs::metadata(&backup).expect("upgrade fixture").modified().expect("upgrade fixture"),
        meta.modified().expect("upgrade fixture")
    );
    scratch.remove();
}
#[test]
fn an_older_recorded_runner_has_its_files_prepared_again_without_a_dump() {
    use std::fs;
    use support::owned;
    use temper_forge_setup::place::File;
    let scratch = support::Scratch::new("runner_upgrade_real");
    let _port = owned::prepared_processes(scratch.path());
    owned::record_versions(scratch.path(), None, Some(b"13.1.0"));
    let configuration = owned::file(scratch.path(), 1, File::RunnerConfiguration);
    let before = fs::read(&configuration).expect("connection");
    fs::remove_file(&configuration).expect("remove configuration");
    let upgraded = owned::prepare(scratch.path(), 1);
    assert!(upgraded.status.success(), "{}", String::from_utf8_lossy(&upgraded.stderr));
    assert_eq!(fs::read(configuration).expect("configuration remade"), before);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}

#[test]
fn temper_box_prints_its_link_and_stops_its_processes() {
    let scratch = support::Scratch::new("box_stop");
    let owned = boxed_example(&scratch);
    let forgejo = owned.forgejo;
    let ended = support::boxed::run_echo_steps(scratch.path(), Some(forgejo), support::boxed::Plan::Stop, BOX_DEADLINE);
    assert_box_streams(&ended, forgejo, scratch.path());
    assert_eq!(ended.forgejo_at_link, Some(200), "Forgejo answered its version when the link came");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    assert_eq!(support::boxed::status(forgejo, "/api/v1/version"), None, "Forgejo is gone");
    scratch.remove();
}
fn boxed_example(scratch: &support::Scratch) -> support::owned::Owned {
    let owned = support::owned::Owned::tier();
    support::owned::lay(scratch.path(), &owned);
    std::fs::write(scratch.path().join("temper.json"), support::owned::example_document(scratch.path(), &owned))
        .expect("box example");
    // A retained group starts only with its started record (temper/docs/design/protocol/setup/box.md, 1).
    for directory in ["box/groups", "box/started"] {
        std::fs::create_dir_all(scratch.path().join(directory)).expect("box group directory");
    }
    support::write_private(&scratch.path().join("box/groups/1"), b"owned\n");
    support::write_private(&scratch.path().join("box/started/1"), b"");
    owned
}
fn assert_box_streams(ended: &support::boxed::Outcome, forgejo: std::net::SocketAddr, data: &std::path::Path) {
    let link = support::printed_link(&ended.stdout).unwrap_or_else(|| {
        panic!("the box's stdout carries the link: {}", String::from_utf8_lossy(&ended.stderr));
    });
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "the sign-in link");
    let alone = |line: &[u8]| line.is_empty() || line.starts_with(b"http://127.0.0.1:");
    assert!(ended.stdout.split(|byte| *byte == b'\n').all(alone), "stdout holds links alone");
    let steps = ["prepare-1: ", "forgejo-1: ", "runner-1: ", "serve: "];
    let api = format!("Forgejo: api http://{forgejo}/api/v1");
    let notices = ["Forge 1 (Forgejo): starting", "Forge 1 (Forgejo): started"];
    let log = std::fs::read_to_string(data.join("box/logs/forgejo-1.log")).expect("Forgejo step log");
    for words in [api.as_str(), "Forgejo: person ada"] {
        let lines: Vec<_> = log.lines().filter(|line| line.ends_with(words)).collect();
        assert_eq!(lines.len(), 1, "hands are logged once: {log}");
        let stamp = lines[0].strip_suffix(words).expect("hands suffix");
        assert_eq!(stamp.len(), 21, "ISO UTC time precedes the hands");
        assert!(stamp.ends_with("Z "));
    }
    let stderr = std::str::from_utf8(&ended.stderr).expect("box diagnostics");
    assert!(
        stderr.lines().all(|line| { steps.iter().any(|step| line.starts_with(step)) || box_terminal_line(line) }),
        "each stderr line names its step or carries prescribed box words: {stderr}"
    );
    let log = support::boxed::box_log(data);
    for words in notices {
        assert_eq!(log.lines().filter(|line| box_notice_words(line) == words).count(), 1, "{log}");
    }
    assert!(!stderr.contains("Forgejo: api ") && !stderr.contains("Forgejo: person "), "{stderr}");
    assert!(stderr.lines().any(|line| line.starts_with("runner-1: ")), "the runner's lines arrive prefixed");
}

// Only a valid UTC time-of-day prefix can precede the box notices.
fn box_notice_words(line: &str) -> &str {
    let bytes = line.as_bytes();
    if bytes.len() >= 10
        && bytes[2] == b':'
        && bytes[5] == b':'
        && &bytes[8..10] == b"Z "
        && [0, 1, 3, 4, 6, 7].iter().all(|index| bytes[*index].is_ascii_digit())
        && &bytes[..2] < b"24".as_slice()
        && &bytes[3..5] < b"60".as_slice()
        && &bytes[6..8] < b"60".as_slice()
    {
        &line[10..]
    } else {
        line
    }
}

fn box_terminal_line(line: &str) -> bool {
    let words = box_notice_words(line);
    line.is_empty()
        || [
            "Open this link in a browser on this machine.",
            "It signs you in as this temper's owner, once, within 2 hours.",
            "Opened after that, it prints a fresh one here.",
        ]
        .contains(&line)
        || line.starts_with("In a terminal: ")
        || words.starts_with("box: ")
}

#[test]
fn temper_box_restarts_a_killed_forgejo_while_the_engine_serves() {
    let scratch = support::Scratch::new("box_kill");
    let owned = boxed_example(&scratch);
    let forgejo = owned.forgejo;
    let kill = support::boxed::Plan::KillForgejo;
    let ended = support::boxed::run(scratch.path(), Some(forgejo), kill, BOX_DEADLINE);
    let (killed, restarted) = ended.forgejo_pids.unwrap_or_else(|| {
        panic!("Forgejo was killed and came back: {}", String::from_utf8_lossy(&ended.stderr));
    });
    assert_ne!(killed, restarted, "another Forgejo process answers");
    assert_eq!(ended.engine_served, Some(true), "the same engine process answered while Forgejo was back");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    scratch.remove();
}

#[test]
fn a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart() {
    use skein_forge::forgejo::pin::VERSION;
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::ctl::Step;

    let scratch = support::Scratch::new("forgejo-from-client");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    for path in &paths {
        fs::create_dir(path).expect("empty first-run directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private first-run directory");
    }
    let flags = support::boxed::echo_steps(deployment_flags(&paths));
    let (cached, sources) = cache_forgejo_binaries(&paths[2]);
    let mut bot = None;
    let mut csv_landing = None;
    let mut older_admin = None;
    for run in ["first", "second"] {
        let client_root = scratch.path().join(run);
        fs::create_dir(&client_root).expect("fresh client root");
        fs::set_permissions(&client_root, fs::Permissions::from_mode(0o700)).expect("private client root");
        let mut steps = forgejo_setup_steps(run);
        let clock = skein_shell::Clock::new();
        let began = clock.now().now;
        steps.push(Step::Mark("first repository ready"));
        if run == "first" {
            // protocol/setup/testing.md, 4: main's push run is taken before the owner's
            // first task; no pull exists yet to have caused this runner line.
            steps.push(Step::Stderr(b" repo is work/hello "));
            steps.extend(support::first_repository::short_waiting_steps(&client_root));
            steps.extend(support::csv_repair::task_steps(&paths, &client_root, began));
        } else {
            steps.extend(support::first_repository::found_steps());
        }
        let ended = support::boxed::run_ctl_with_llm(
            scratch.path(),
            flags.clone(),
            &client_root,
            steps,
            BOX_DEADLINE,
            |_| {
                let evidence = support::first_repository::live_from_hands(&paths[0], &paths[1], &client_root);
                support::first_repository::probe_admin_handover(&paths, &client_root, &mut older_admin);
                evidence
            },
            support::csv_repair::llm(&paths[0]),
        );
        eprintln!("{run} Forgejo setup run: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
        let stderr = String::from_utf8_lossy(&ended.stderr);
        assert_eq!(ended.exit, Some(Exit::Code(0)), "{run}: {stderr}");
        assert_eq!(ended.left, Some(Vec::new()), "{run}: no process outlived the box: {stderr}");
        let setup_calls = if run == "first" { 7 } else { 4 };
        bot = Some(assert_forgejo_setup(run, &ended.calls[..setup_calls], &stderr, bot, &paths[1]));
        for call in &ended.calls[setup_calls..] {
            assert_eq!(
                call.exit,
                Some(Exit::Code(0)),
                "stdout {}; stderr {}; box {stderr}",
                String::from_utf8_lossy(&call.stdout),
                String::from_utf8_lossy(&call.stderr)
            );
        }
        let evidence = ended.checked.expect("the live forge and task reads ran");
        support::first_repository::assert_task(run, &ended.calls[setup_calls..], &stderr, &evidence);
        let panel = support::ctl::json(&evidence.panel);
        assert_eq!(panel["step"], "landed");
        let merge = panel["merge"].as_str().expect("landed merge commit");
        assert_eq!(merge.len(), 40);
        let identity = (panel["head"].clone(), panel["merge"].clone());
        if run == "first" {
            support::csv_repair::assert_briefs(ended.model.as_ref().expect("model observations"));
            assert_eq!(panel["repairs"]["used"], 1);
            let pending = ended
                .calls
                .iter()
                .filter_map(|call| serde_json::from_slice::<serde_json::Value>(&call.stdout).ok())
                .find(|panel| panel["gates"][0]["verdict"] == "pending")
                .expect("pending review at head");
            assert_eq!(panel["head"], pending["head"], "land exactly the reviewed head");
            assert_eq!(panel["gates"][0]["verdict"], "approved");
            assert!(evidence.producer.contains("--csv"), "the coder wrote widget's CSV implementation");
            csv_landing = Some(identity);
        } else {
            assert_eq!(Some(identity), csv_landing, "the same landing survives restart");
        }
        let support::first_repository::Evidence { address, version, .. } = evidence;
        let version = version.unwrap_or_else(|why| panic!("{run}: live Forgejo version: {why:?}; box: {stderr}"));
        assert_eq!(version["version"].as_str().expect("version").split('+').next(), Some(VERSION));
        support::assert_one_printed_link(run, &ended.stdout, &ended.stderr);
        assert_eq!(support::boxed::status(address, "/api/v1/version"), None, "Forgejo stopped with its box");
        assert_cache_untouched(&cached, &sources);
        if run == "first" {
            replace_admin_handover(&paths);
        }
        let wrote = format!("First run: wrote the configuration, {}\n", paths[0].join("temper.json").display());
        assert_eq!(
            support::boxed::box_log(&paths[1]).contains(&wrote),
            run == "first",
            "only the first box writes configuration"
        );
    }
    scratch.remove();
}

fn assert_cache_untouched(cached: &[std::path::PathBuf; 2], sources: &[std::path::PathBuf; 2]) {
    for (path, source) in cached.iter().zip(sources) {
        assert_cached_binary_unchanged(path, source);
        let part =
            path.with_file_name(format!("{}.part", path.file_name().expect("binary name").to_str().expect("UTF-8")));
        assert!(!part.exists(), "no partial download");
    }
}

fn replace_admin_handover(paths: &[std::path::PathBuf; 3]) {
    let prepared = support::story_command::standalone(
        std::process::Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["forge", "prepare"])
            .args(deployment_flags(paths))
            .arg("1")
            .current_dir(&paths[1]),
        "the replacement admin handover",
        BOX_DEADLINE,
    );
    assert!(prepared.status.success(), "{}", String::from_utf8_lossy(&prepared.stderr));
}

#[test]
fn a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{self, Plan};

    let scratch = support::Scratch::new("forgejo-plan-change");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    let mut issuer = Some(forge_land::fake_deployment(&paths));
    let flags = support::boxed::echo_steps(deployment_flags(&paths));
    let (cached, sources) = cache_forgejo_binaries(&paths[2]);
    let clock = skein_shell::Clock::new();
    let addition = format!("Plan change at {}-{}", std::process::id(), clock.now().now.as_nanos());
    let words = format!("Please append this line to work/hello README.md: {addition}");
    let mut bot = None;
    let mut landed = None;
    let mut ended_tasks = None;
    for run in ["first", "second"] {
        let first = run == "first";
        let client = scratch.path().join(run);
        fs::create_dir(&client).expect("fresh client root");
        fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client root");
        let mut steps = forgejo_setup_steps(run);
        let setup_calls = if first { 7 } else { 4 };
        let (account_at, found_at) = plan_change_steps(&mut steps, first, setup_calls, &words);
        let scripts = if first {
            forge_land::plan_scripts(&Plan {
                words: words.as_bytes(),
                line: addition.as_bytes(),
                repository: temper_engine_domain_forge::tag(3, 1),
                readme: b"# hello\n\n",
            })
        } else {
            // A fresh fake has no request cursor from the old process. Its
            // first turn answers Close's finish request with text, which parks
            // the Open run. Its system cue remains the original task's words.
            forge_land::restart_script(words.as_bytes())
        };
        let issuer = issuer.take().unwrap_or_else(|| support::sign_in::restart_issuer(&paths[0]));
        let clock = skein_shell::Clock::new();
        let began = clock.now().now;
        let (outcome, seen) = support::boxed::run_ctl_with_face(
            scratch.path(),
            flags.clone(),
            &client,
            steps,
            BOX_DEADLINE,
            || support::first_repository::live(&paths[0], &paths[1], &client),
            forge_land::plan_face(issuer, scripts, first),
        );
        eprintln!("{run} plan Forgejo box: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        eprintln!(
            "{run} peer: {} requests; answers {:?}; schemes {:?}",
            seen.requests.len(),
            seen.answers,
            seen.schemes
        );
        assert_plan_box(run, first, &outcome, &stderr);
        assert!(outcome.calls.len() > found_at + 3, "all story reads ran: {stderr}");
        bot = Some(assert_forgejo_setup(run, &outcome.calls[..setup_calls], &stderr, bot, &paths[1]));
        support::sign_in::assert_plan_account(&outcome.calls[setup_calls], &outcome.calls[account_at]);
        let evidence = outcome.checked.expect("live check ran");
        support::first_repository::assert_task(run, &outcome.calls[found_at..found_at + 3], &stderr, &evidence);
        forge_land::assert_landed_chat_pull(run, &evidence);
        let task = String::from_utf8(outcome.calls[found_at + 1].stdout.clone()).expect("landed task UTF-8");
        let identity = forge_land::landed_identity(&task, &evidence.page);
        let closing_at = outcome.calls.len() - if first { 0 } else { 4 };
        let (tasks_at, tasks) = forge_land::plan_ended_tasks(&outcome.calls[..closing_at]);
        if first {
            landed = Some(identity);
            ended_tasks = Some(tasks);
        } else {
            assert_eq!(Some(identity), landed, "same pull and merge commit");
            assert_eq!(Some(tasks), ended_tasks, "no second change or producer");
            let ended = support::ctl::json(&outcome.calls[closing_at + 1]);
            assert_eq!(ended["met"], true);
            assert_eq!(ended["task"]["chip"]["phase"], "ended");
            forge_land::assert_no_live_roots(&support::ctl::json(&outcome.calls[closing_at + 2]));
            let unknown = &outcome.calls[closing_at + 3];
            assert_eq!(unknown.exit, Some(Exit::Code(2)), "no task 5: {stderr}");
            assert_eq!(unknown.stderr.trim_ascii_end(), b"{\"error\":\"unknown\"}", "no task 5: {stderr}");
        }
        forge_land::assert_reply(&outcome.calls[found_at + 4..tasks_at], first);
        // One reviewer turn follows CI. Four coder turns include the workflow read;
        // three chat turns delegate, wait, then reply after the procedure ends.
        // After the restart, the chat's lost attempt answers again, then Close's
        // finish request takes its last text.
        forge_land::assert_requests(&seen, if first { 8 } else { 2 }, words.as_bytes(), first);
        support::assert_one_printed_link(run, &outcome.stdout, &outcome.stderr);
        assert_eq!(support::boxed::status(evidence.address, "/api/v1/version"), None);
        assert_eq!(
            evidence.version.expect("live Forgejo")["version"].as_str().expect("version").split('+').next(),
            Some(skein_forge::forgejo::pin::VERSION)
        );
        let wrote = format!("First run: wrote the configuration, {}\n", paths[0].join("temper.json").display());
        assert!(!support::boxed::box_log(&paths[1]).contains(&wrote), "init wrote configuration before either box");
        assert_cache_untouched(&cached, &sources);
    }
    scratch.remove();
}

// protocol/cluster.md, 6: the description's first process stories.
struct ClusterJudge {
    members: support::cluster::Members,
    deadline: skein_lib::Time,
    workers: bool,
}
impl skein_world::Referee<support::cluster::Root> for ClusterJudge {
    fn act(&mut self, now: skein_lib::Time, hosts: &mut [support::cluster::Root]) {
        if self.members.is_up() {
            if self.workers {
                for worker in ["w1", "w2"] {
                    assert_eq!(
                        self.members.capture(worker, "engine"),
                        self.members.capture("engine", "workers"),
                        "{worker} linked to this engine"
                    );
                }
            }
            self.members.stop();
        }
        self.members.act(now, hosts);
    }
    fn observe(&mut self, now: skein_lib::Time, hosts: &[support::cluster::Root]) {
        self.members.observe(now, hosts);
        if let Some(stopped) = self.members.stopped() {
            if self.workers {
                assert_eq!(
                    stopped.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
                    ["w2", "w1", "engine", "llm", "forge"]
                );
            } else {
                let [engine, llm, forge] = stopped else { panic!("engine, llm and forge stopped") };
                assert_eq!(engine.name, "engine");
                assert_eq!(llm.name, "llm");
                assert_eq!(forge.name, "forge");
                engine_lines(engine);
            }
            for member in stopped {
                clean_cluster_exit(member);
            }
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: skein_lib::Time) -> Option<String> {
        self.members.overdue(now).or_else(|| {
            (now >= self.deadline && !self.members.passed()).then(|| "cluster did not settle within WITHIN".into())
        })
    }
    fn passed(&self) -> bool {
        self.members.passed()
    }
}
fn cluster_story(scratch: &std::path::Path, workers: bool) {
    let cluster = support::cluster::Cluster::read(scratch);
    let (mut members, hosts) = cluster.members();
    members.up(if workers { &["w1", "w2"] } else { &["engine"] });
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(ClusterJudge {
        members,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        workers,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    let outcome = world.run(&clock, support::WITHIN);
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "every cluster root settled");
}
fn clean_cluster_exit(stopped: &support::cluster::Stopped) {
    assert_eq!(stopped.exit, Some(Exit::Code(0)), "{}: {}", stopped.name, String::from_utf8_lossy(&stopped.stderr));
    assert!(!stopped.killed, "{} stopped before STOP_SECONDS", stopped.name);
    assert!(stopped.survivors.is_empty(), "{} left no survivor", stopped.name);
}
#[test]
fn an_engine_alone_prints_its_lines_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("cluster_engine_alone");
    cluster_story(scratch.path(), false);
    scratch.remove();
}
fn engine_lines(engine: &support::cluster::Stopped) {
    let stderr = std::str::from_utf8(&engine.stderr).expect("engine lines");
    let lines: Vec<_> = stderr.lines().collect();
    let [_, _, _] = lines.as_slice() else { panic!("exactly the three listening lines") };
    let clients: std::net::SocketAddr = lines
        .iter()
        .find_map(|line| line.strip_prefix("listening clients "))
        .expect("clients line")
        .parse()
        .expect("clients address");
    let workers: std::net::SocketAddr = lines
        .iter()
        .find_map(|line| line.strip_prefix("listening workers "))
        .expect("workers line")
        .parse()
        .expect("workers address");
    let webhooks: std::net::SocketAddr = lines
        .iter()
        .find_map(|line| line.strip_prefix("listening webhooks "))
        .expect("webhooks line")
        .parse()
        .expect("webhooks address");
    for address in [clients, workers, webhooks] {
        assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_ne!(address.port(), 0, "a bound port");
    }
    assert_ne!(clients.port(), workers.port(), "distinct listeners");
    assert_ne!(clients.port(), webhooks.port(), "distinct listeners");
    assert_ne!(workers.port(), webhooks.port(), "distinct listeners");
    let stdout = std::str::from_utf8(&engine.stdout).expect("link text");
    let link = stdout.strip_suffix('\n').expect("terminated sign-in link");
    assert!(!link.contains('\n'), "exactly one stdout line");
    let secret = link
        .strip_prefix(&format!("http://{clients}/jig/v1/sign-in/link?secret="))
        .expect("sign-in link at the clients listener");
    assert!(!secret.is_empty(), "a sign-in secret");
}

#[test]
fn a_mixed_document_exits_1_naming_its_path() {
    use skein_world::end_to_end::{Binary, Command, Expectation, Mode};
    struct Refusal {
        deadline: skein_lib::Time,
        settled: bool,
    }
    impl skein_world::Referee<Binary> for Refusal {
        fn observe(&mut self, _now: skein_lib::Time, hosts: &[Binary]) {
            self.settled = hosts.iter().all(skein_shell::Host::is_empty);
        }
        fn next_deadline(&self) -> Option<skein_lib::Time> {
            Some(self.deadline)
        }
        fn overdue(&self, now: skein_lib::Time) -> Option<String> {
            (now >= self.deadline && !self.settled).then(|| "mixed document did not exit".into())
        }
        fn passed(&self) -> bool {
            self.settled
        }
    }
    let scratch = support::Scratch::new("cluster_mixed_document");
    let cluster = support::cluster::Cluster::documents(
        scratch.path(),
        &[("forge", "address", "127.0.0.1:3"), ("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    );
    let path = cluster.dir("engine").join("config/temper.json");
    let mut engine: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("engine document")).expect("engine JSON");
    let worker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cluster.dir("w1").join("config/temper.json")).expect("worker document"))
            .expect("worker JSON");
    engine["workers"] = worker["workers"].clone();
    std::fs::write(&path, serde_json::to_vec(&engine).expect("mixed JSON")).expect("mixed document");
    let directory = cluster.dir("engine");
    let binary = Binary::start_with_tree(
        Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: vec![
                "serve".into(),
                "--config".into(),
                directory.join("config").into(),
                "--data".into(),
                directory.join("data").into(),
                "--cache".into(),
                directory.join("cache").into(),
            ],
            environment: Vec::new(),
            directory,
        },
        Mode::Pipes,
        65_536,
        Expectation::Supervise,
        false,
    )
    .expect("mixed serve starts");
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(Refusal {
        deadline: clock.now().now.saturating_add(support::WITHIN),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || binary);
    let outcome = world.run(&clock, support::WITHIN);
    let [binary] = outcome.procs.as_slice() else { panic!("one serve") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(1)));
    assert_eq!(binary.stderr(), b"temper serve: workers: an engine alone takes no local worker, one plaintext loopback listener and workers with distinct digest paths\n");
    scratch.remove();
}

#[test]
fn the_cluster_stops_in_reverse_each_exiting_0() {
    let scratch = support::Scratch::new("cluster_reverse_stop");
    cluster_story(scratch.path(), true);
    scratch.remove();
}

// The worker retains the owned example's limits.
fn prepare_owned_worker(members: &mut support::cluster::Members) {
    let directory = members.prepare("w1");
    let configuration = directory.join("config");
    let agent_path = configuration.join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/examples/owned/agent.json")).expect("owned agent");
    agent["programs"]["claude_code"]["directory"] = directory.join("programs").to_str().expect("programs").into();
    support::write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    let example: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/examples/owned.json")).expect("owned example");
    let mut document = serde_json::json!({
        "workers": example["workers"],
        "application": {"workers": example["application"]["workers"], "limits": example["application"]["limits"]}
    });
    let worker = &mut document["workers"][0];
    worker["engine"] = serde_json::json!({"socket": members.capture("engine", "workers").expect("engine listener")});
    worker["name"] = "w1".into();
    worker["secret"] = "secrets/w1.secret".into();
    worker["agent"] = serde_json::json!({
        "program": skein_scratch::paths::binary("temper-box"), "configuration": agent_path,
        "arguments": ["internal", "agent", agent_path]
    });
    let arguments = [
        "--config".to_owned(),
        configuration.to_str().expect("config path").to_owned(),
        "--data".to_owned(),
        directory.join("data").to_str().expect("data path").to_owned(),
        "--cache".to_owned(),
        directory.join("cache").to_str().expect("cache path").to_owned(),
    ];
    support::write_private(&configuration.join("temper.json"), &serde_json::to_vec(&document).expect("worker JSON"));
    let arguments: Vec<Box<[u8]>> = arguments.iter().map(|s| s.as_bytes().into()).collect();
    let (directories, _) = temper_box_shell::directories::resolve(&arguments, &[], b"/").expect("worker directories");
    let operations =
        temper_box_shell::operations_in(&[], b"/", &directories, &serde_json::to_vec(&document).expect("worker JSON"))
            .expect("owned worker starts at its declared figures");
    assert!(operations <= 32_768, "Linux IORING_MAX_ENTRIES");
}

enum OwnedWorkerPhase {
    Engine,
    Worker,
    Linked,
    Stopping,
}
struct OwnedWorkerJudge {
    members: support::cluster::Members,
    deadline: skein_lib::Time,
    phase: OwnedWorkerPhase,
}
impl skein_world::Referee<support::cluster::Root> for OwnedWorkerJudge {
    fn act(&mut self, now: skein_lib::Time, hosts: &mut [support::cluster::Root]) {
        if self.members.is_up() {
            match self.phase {
                OwnedWorkerPhase::Engine => {
                    let directory = self.members.prepare("engine");
                    let path = directory.join("config/temper.json");
                    let mut engine: serde_json::Value =
                        serde_json::from_slice(&std::fs::read(&path).expect("engine document")).expect("engine JSON");
                    // Negotiate the owned worker's full terms, including its turn bounds.
                    let example: serde_json::Value =
                        serde_json::from_str(include_str!("../../../docs/examples/owned.json")).expect("owned example");
                    engine["engine"]["limits"] = example["engine"]["limits"].clone();
                    engine["application"]["limits"] = example["application"]["limits"].clone();
                    // The example's 186 s worker stop bound must fit the engine's 240 s grace.
                    engine["engine"]["limits"]["overrides"]["fleet.grace_ms"] = 240_000.into();
                    support::write_private(&path, &serde_json::to_vec(&engine).expect("engine JSON"));
                    self.members.up(&["engine"]);
                    self.phase = OwnedWorkerPhase::Worker;
                }
                OwnedWorkerPhase::Worker => {
                    prepare_owned_worker(&mut self.members);
                    self.members.up(&["w1"]);
                    self.phase = OwnedWorkerPhase::Linked;
                }
                OwnedWorkerPhase::Linked => {
                    assert_eq!(self.members.capture("w1", "engine"), self.members.capture("engine", "workers"));
                    assert_eq!(self.members.linked("w1"), 1, "the owned worker linked");
                    self.members.stop();
                    self.phase = OwnedWorkerPhase::Stopping;
                }
                OwnedWorkerPhase::Stopping => {}
            }
        }
        self.members.act(now, hosts);
    }
    fn observe(&mut self, now: skein_lib::Time, hosts: &[support::cluster::Root]) {
        self.members.observe(now, hosts);
        if let Some(stopped) = self.members.stopped() {
            assert_eq!(stopped.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["w1", "engine", "llm", "forge"]);
            for member in stopped {
                clean_cluster_exit(member);
            }
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: skein_lib::Time) -> Option<String> {
        self.members.overdue(now).or_else(|| {
            (now >= self.deadline && !self.members.passed()).then(|| "owned worker did not settle within WITHIN".into())
        })
    }
    fn passed(&self) -> bool {
        self.members.passed()
    }
}
#[test]
fn a_worker_alone_with_the_owned_examples_worker_links_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("owned_worker_alone_ring");
    let (mut members, hosts) = support::cluster::Cluster::read(scratch.path()).members();
    members.up(&["llm"]);
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(OwnedWorkerJudge {
        members,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        phase: OwnedWorkerPhase::Engine,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    let outcome = world.run(&clock, support::WITHIN);
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "every cluster root settled");
    scratch.remove();
}

// Both the idle threshold and ordinary time allowance hold an open chat's slot.
// Past the story's deadline, a run that parks has already failed it.
const CHAT_HOLDS_ITS_SLOT: Duration = support::WITHIN.saturating_mul(2);

// A lost link heals: a stall past the 1 s silence loses it. The worker's contact
// grace, graces.grace_ms 1000 (temper-service worker_settings.rs:145;
// worker-link component.rs:629), outlasts two redials (125–250 ms, doubling:
// backoff.rs:16; profile/small.rs:239); a re-hello clears it (component.rs:455).
// The engine keeps that channel's attempts adrift for fleet.grace_ms 3000
// (jig-domain-fleet channel.rs:250); re-hello lists them back (attempt.rs:49),
// so nothing is placed twice.
#[test]
fn a_task_runs_on_each_worker() {
    cluster_chats("cluster_each_worker", ClusterStory::Each);
}

/// A task on each worker, on the deployment's binaries (protocol/cluster.md, 6).
#[test]
fn a_task_runs_on_each_worker_of_the_deployments_binaries() {
    cluster_chats_with(
        "cluster_binaries_each_worker",
        ClusterStory::Each,
        support::cluster::Cluster::deployment_binaries,
    );
}

// A waiting reply is uncommitted: killing w1 loses it. The retry on w2
// resumes without an assistant message, so the same cue replies and waits
// again. Only after close/end is that reply committed, exactly once; the
// old attempt ends lost and the replacement parks. This covers loss
// and retry; the hung-worker story will test fencing a surviving old run.
#[test]
fn a_worker_killed_mid_run_is_lost_and_its_run_answered_once_on_another() {
    cluster_chats("cluster_killed", ClusterStory::Kill);
}

// protocol/cluster.md, 6: heal within fleet grace, then fence a surviving run.
// Continue follows the connected:false read immediately. CLOCK_MONOTONIC keeps
// advancing during STOP (skein-shell clock.rs:17), but w1 arms its contact
// grace only on detecting its own loss after CONT (jig-protocol-worker-link
// component.rs:169, :622). Its first redial is 125–250 ms (backoff.rs:15;
// jig-protocol-config profile/small.rs:239); its re-hello must
// reach the engine within the 3000 ms fleet grace (jig-domain-fleet
// attempt.rs:505). This is a margin, checked by the unchanged attempt below.
#[test]
fn a_worker_hung_past_the_grace_is_fenced_and_links_again() {
    cluster_chats("cluster_hung", ClusterStory::Hung);
}

// Jig admits only with an available placement (jig-domain-core
// routing/run_preparation.rs:466–492, run_choice.rs:73–77): the second task
// is due, unclaimed and unplaced; no attempt exists to be placed twice.
#[test]
fn a_worker_started_later_takes_the_waiting_task() {
    cluster_chats("cluster_started_later", ClusterStory::Later);
}

#[test]
fn a_worker_restarted_on_its_directories_links_at_its_position() {
    cluster_chats("cluster_restarted", ClusterStory::Restart);
}

#[test]
fn an_unexpected_member_exit_is_reported_after_its_client_settles() {
    let failure = std::panic::catch_unwind(|| cluster_chats("cluster_unexpected", ClusterStory::Unexpected))
        .expect_err("unexpected member fails the story");
    let message = failure.downcast_ref::<String>().expect("member diagnostic");
    assert!(message.contains("w1 exited unexpectedly: Some(Signal(9))"), "{message}");
    assert!(message.contains("linked w1 "), "stderr tail: {message}");
    assert!(!message.contains("EBUSY") && !message.contains("ResourceBusy"), "{message}");
}

fn cluster_chats(case: &str, story: ClusterStory) {
    cluster_chats_with(case, story, |_| {});
}

fn cluster_chats_with(case: &str, story: ClusterStory, shape: fn(&mut support::cluster::Cluster)) {
    let scratch = support::Scratch::new(case);
    let mut cluster = support::cluster::Cluster::read(scratch.path());
    shape(&mut cluster);
    if matches!(story, ClusterStory::Kill | ClusterStory::Hung) {
        cluster.record_llm();
    }
    let settings = cluster.settings_file();
    let key = cluster.dir("llm").join("llm.key");
    let (mut members, hosts) = cluster.members();
    members.up(if story == ClusterStory::Each { &["w1", "w2"] } else { &["w1"] });
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(ClusterChatJudge {
        members,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        settings,
        key,
        client: scratch.path().join("client"),
        stage: ClusterChatStage::Start,
        tasks: Vec::new(),
        workers: std::collections::BTreeSet::new(),
        story,
        reading: None,
        checking: std::collections::VecDeque::new(),
        last_read: String::new(),
        lost_attempt: None,
        healed_attempt: None,
        linked_before: 0,
        fenced: None,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(support::cluster::Root::client);
    let outcome = world.run(&clock, support::WITHIN);
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "every cluster root settled");
    eprintln!("cluster chats settled in {:?}", outcome.end.saturating_since(outcome.start));
    scratch.remove();
}

// bin/temper-box/tests/fixtures/cluster has two chats, one per slot, and two lives in the restart story.
const CLUSTER_CHATS: usize = 2;
const FIRST_LIFE: usize = 0;
const SECOND_LIFE: usize = 1;
const RESTART_LINKS: usize = 2;
const REPLY_ONCE: usize = 1;
const FIRST_LINK: usize = 1;
const LOST_AND_RETRY: usize = 2; // the killed run and its replacement
const KILL_SIGNAL: u32 = 9; // Linux SIGKILL, as reported by wait
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClusterStory {
    Each,
    Unexpected,
    Kill,
    Hung,
    Later,
    Restart,
}
#[derive(Debug)]
enum ClusterChatStage {
    Start,
    Created,
    Replied,
    WaitingSlot,
    Crashed,
    Armed,
    Killed,
    HealSilence,
    HealRelink,
    FenceReady,
    FenceReplacement,
    FenceSnapshot,
    FenceRelink,
    FenceCheck,
    StoppedOne,
    Relinking,
    Closed,
    Verifying,
    Stopping,
}
struct ClusterChatJudge {
    members: support::cluster::Members,
    deadline: skein_lib::Time,
    settings: std::path::PathBuf,
    key: std::path::PathBuf,
    client: std::path::PathBuf,
    stage: ClusterChatStage,
    tasks: Vec<String>,
    workers: std::collections::BTreeSet<String>,
    story: ClusterStory,
    reading: Option<ClusterRead>,
    checking: std::collections::VecDeque<String>,
    last_read: String,
    lost_attempt: Option<u64>,
    healed_attempt: Option<u64>,
    linked_before: usize,
    fenced: Option<(Vec<serde_json::Value>, Vec<serde_json::Value>)>,
}
impl ClusterChatJudge {
    fn queue(&self, host: &mut support::cluster::Root, calls: Vec<support::ctl::Call>) {
        self.queue_steps(host, calls.into_iter().map(support::ctl::Step::Run).collect());
    }
    fn queue_steps(&self, host: &mut support::cluster::Root, steps: Vec<support::ctl::Step>) {
        let link = format!("http://{}", self.members.capture("engine", "link").expect("engine link"));
        let mut client = support::ctl::Calls::linked(&link, &self.client);
        for step in steps {
            client.push(step);
        }
        let support::cluster::Root::Client { calls, .. } = host else { panic!("client root") };
        *calls = Some(Box::new(client));
    }
    fn slot_hold(&self) {
        // Check the fixture's waiting and time overrides before applying them to current charters.
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&self.settings).expect("cluster settings FILE"))
                .expect("settings JSON");
        let charter = &settings["items"]
            .as_array()
            .expect("settings items")
            .iter()
            .find(|item| item["setting"] == serde_json::json!({"kind":"charter","number":1}))
            .expect("chat charter setting")["set"];
        assert!(
            Duration::from_nanos(charter["waiting"].as_u64().expect("chat waiting")) >= CHAT_HOLDS_ITS_SLOT,
            "bin/temper-box/tests/fixtures/cluster/settings.json: charter 1 waiting must hold its slot past the story deadline"
        );
        let time = charter["time"].as_u64().expect("chat time");
        let reserve = charter["reserve_time"].as_u64().expect("chat reserve_time");
        assert!(
            time.checked_sub(reserve).is_some_and(|ordinary| Duration::from_nanos(ordinary) >= CHAT_HOLDS_ITS_SLOT),
            "bin/temper-box/tests/fixtures/cluster/settings.json: charter 1 time less its reserve must hold its slot past the story deadline"
        );
    }
    fn begin(&mut self, client: &mut support::cluster::Root) {
        use support::ctl::{Call, PROJECT, Step};
        self.slot_hold();
        let settings = self.settings.clone();
        self.queue_steps(
            client,
            vec![
                Step::Run(Call::sign_in()),
                Step::Run(Call::new(&["settings", "show", "--json"])),
                Step::Run(Call::new(&["settings", "show", "charter", "1", "--json"])),
                Step::Run(Call::new(&["settings", "show", "charter", "2", "--json"])),
                Step::Run(Call::new(&["settings", "show", "template", "1", "--json"])),
                Step::Then(Box::new(move |calls| {
                    cluster_settings(&settings, &calls[2..]);
                    support::ctl::settings_change(&calls[1], &settings, "cluster-settings")
                })),
                Step::Run(Call::new(&[
                    "account",
                    "add-key",
                    "1",
                    self.key.to_str().expect("LLM key path"),
                    "--deployment",
                    "--key",
                    "cluster-llm",
                ])),
                Step::Run(Call::new(&["account", "list", "--until", "usable"])),
                Step::Run(Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    PROJECT,
                    "--key",
                    "first",
                    "cluster-first-chat",
                    "--json",
                ])),
            ],
        );
        self.stage = ClusterChatStage::Created;
    }
    fn create_second(&mut self, client: &mut support::cluster::Root) {
        use support::ctl::{Call, PROJECT};
        self.queue(
            client,
            vec![Call::new(&[
                "chat",
                "start",
                "--project",
                PROJECT,
                "--key",
                "second",
                "cluster-second-chat",
                "--json",
            ])],
        );
        self.stage = ClusterChatStage::Created;
    }
    fn current(&self) -> &str {
        self.checking.front().or_else(|| self.tasks.last()).expect("current chat")
    }
    fn poll(&self, client: &mut support::cluster::Root) {
        use support::ctl::Call;
        let task = self.current();
        let calls = if matches!(
            self.stage,
            ClusterChatStage::Verifying | ClusterChatStage::FenceSnapshot | ClusterChatStage::FenceCheck
        ) {
            vec![Call::new(&[
                "task",
                "show",
                task,
                "--activity",
                &temper_client_tools::limits::ROWS.to_string(),
                "--json",
            ])]
        } else if matches!(self.stage, ClusterChatStage::Replied) {
            // The live reply watch returns a task; read.waiting() still checks its exact work and placement.
            vec![
                Call::new(&["task", "show", task, "--until", "replied", "--json"]),
                Call::new(&["account", "list", "--json"]),
            ]
        } else {
            vec![Call::new(&["task", "show", task, "--json"]), Call::new(&["account", "list", "--json"])]
        };
        self.queue(client, calls);
    }
    fn read(&mut self, client: &mut support::cluster::Root, ends: &[support::Ended]) -> Option<ClusterRead> {
        use support::ctl::{Call, json};
        if self.reading.as_ref().is_some_and(|read| read.turns) {
            let mut read = self.reading.take().expect("turn reads");
            let turns = read.rows.iter_mut().filter(|row| row["event"]["owner"]["source"]["line"]["turn"].is_object());
            for (row, ended) in turns.zip(ends) {
                let turn = json(ended);
                complete_cluster_json(&turn);
                assert!(turn["turn"]["decoded"].is_object(), "committed turn decoded: {turn}");
                row["event"]["owner"]["decoded"] = turn;
            }
            return Some(read);
        }
        let (mut read, page) = match ends {
            [task, accounts] => {
                let task = json(task);
                let task = if matches!(self.stage, ClusterChatStage::Replied) { task["task"].clone() } else { task };
                return Some(ClusterRead::new(task, json(accounts)));
            }
            [activity] if self.reading.is_some() => (self.reading.take().expect("continued activity"), json(activity)),
            [packet] => {
                let packet = json(packet);
                (ClusterRead::new(packet["task"].clone(), serde_json::Value::Null), packet["activity"].clone())
            }
            _ => panic!("task/accounts, task with activity, or continued activity"),
        };
        complete_cluster_json(&page);
        read.rows.extend(page["rows"].as_array().expect("activity rows").iter().cloned());
        let task = self.current();
        if let Some(before) = page["before"]["value"].as_str() {
            self.queue(client, vec![Call::new(&["task", "activity", task, "--before", before, "--json"])]);
        } else {
            let calls: Vec<_> = read
                .rows
                .iter()
                .filter_map(|row| {
                    let turn = row.pointer("/event/owner/source/line/turn")?;
                    let attempt = turn["attempt"].as_u64().expect("turn attempt");
                    let number = turn["turn"].as_u64().expect("turn number");
                    Some(Call::new(&["task", "turn", task, &format!("{attempt}.{number}"), "--json"]))
                })
                .collect();
            if calls.is_empty() {
                return Some(read);
            }
            self.queue(client, calls);
            read.turns = true;
        }
        self.reading = Some(read);
        None
    }
    fn advance_read(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        self.last_read = read.owner().to_string();
        match self.stage {
            ClusterChatStage::WaitingSlot => {
                if read.owner()["phase"] == serde_json::json!({"active": "due"})
                    && read.owner()["details"]["attempt"].is_null()
                    && read.owner()["details"]["observation"].is_null()
                {
                    self.members.up(&["w2"]);
                    self.stage = ClusterChatStage::Replied;
                }
                self.poll(client);
            }
            ClusterChatStage::Relinking => {
                if self.members.linked("w1") == RESTART_LINKS && read.connected("w1") {
                    self.create_second(client);
                } else {
                    self.poll(client);
                }
            }
            ClusterChatStage::Replied => self.replied(client, read),
            ClusterChatStage::HealSilence
            | ClusterChatStage::HealRelink
            | ClusterChatStage::FenceReady
            | ClusterChatStage::FenceReplacement
            | ClusterChatStage::FenceSnapshot
            | ClusterChatStage::FenceRelink
            | ClusterChatStage::FenceCheck => self.hung_read(client, read),
            ClusterChatStage::Armed => {
                if self.members.is_up() && self.members.linked("w2") >= FIRST_LINK && read.connected("w2") {
                    self.members.kill("w1");
                    self.stage = ClusterChatStage::Killed;
                }
                self.poll(client);
            }
            ClusterChatStage::Killed => {
                if self.members.ended("w1")
                    && read.worker() == Some("w2")
                    && read.waiting()
                    && Some(read.attempt()) != self.lost_attempt
                {
                    self.close(client, false);
                } else {
                    self.poll(client);
                }
            }
            ClusterChatStage::Verifying => self.verify(client, read),
            ClusterChatStage::Start
            | ClusterChatStage::Created
            | ClusterChatStage::StoppedOne
            | ClusterChatStage::Crashed
            | ClusterChatStage::Closed
            | ClusterChatStage::Stopping => unreachable!("read stage"),
        }
    }
    fn verify(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        if matches!(self.story, ClusterStory::Kill | ClusterStory::Hung) {
            // Keep causal client evidence beside the fake's query records on failure.
            // Neither includes the linked client's sign-in link or engine stdout.
            let path = self.members.dir("llm").join(format!("task-{}-attempt-{}.json", self.current(), read.attempt()));
            let evidence = serde_json::json!({"task": &read.task, "activity": &read.rows});
            support::write_private(&path, &serde_json::to_vec(&evidence).expect("chat evidence"));
            eprintln!("cluster close evidence: {}", path.display());
        }
        // The chat is ended by its requester (domain/agent/charter.md, 5): its person's Close
        // asks its Open run to wind down; the final text parks with no task result.
        let cancelled = &read.owner()["details"]["ending"]["cancelled"];
        assert_eq!(cancelled["by"]["person"], 1, "chat closed by its person: {}", read.task);
        assert!(cancelled["result"].is_null(), "a closed chat keeps no finish's result: {}", read.task);
        let text = if self.tasks.first().is_some_and(|task| task == self.current())
            || (self.story == ClusterStory::Hung && self.tasks.get(1).is_some_and(|task| task == self.current()))
        {
            "First worker replied."
        } else {
            "Second worker replied."
        };
        assert_eq!(read.replies(text), REPLY_ONCE, "reply once across complete committed turns");
        if self.story == ClusterStory::Kill
            || (self.story == ClusterStory::Hung && self.tasks.get(1).is_some_and(|task| task == self.current()))
        {
            self.loss(read);
        }
        if self.story == ClusterStory::Hung && self.tasks.len() == FIRST_LINK {
            let attempt = self.healed_attempt.expect("healed attempt");
            assert_eq!(read.attempt(), attempt, "heal retains its attempt");
            let lines: Vec<_> = read.rows.iter().map(|row| &row["event"]["owner"]["source"]["line"]).collect();
            let began: Vec<_> = lines.iter().filter(|line| line["run_began"].is_object()).collect();
            assert_eq!(began.len(), REPLY_ONCE, "heal places exactly once");
            assert_eq!(began[0]["run_began"]["attempt"], attempt);
            let ends: Vec<_> = lines.iter().filter(|line| line["run_ended"].is_object()).collect();
            assert_eq!(ends.len(), REPLY_ONCE, "same healed attempt ended once");
            assert_eq!(ends[0]["run_ended"]["attempt"], attempt);
            // The admitted finish request makes the Open run's next text park (jig's domain/hosts/README.md, 7).
            assert_eq!(ends[0]["run_ended"]["end"], "parked");
            self.checking.pop_front();
            self.queue(
                client,
                vec![support::ctl::Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    support::ctl::PROJECT,
                    "--key",
                    "fence",
                    "cluster-first-chat",
                    "--json",
                ])],
            );
            self.stage = ClusterChatStage::Created;
            return;
        }
        self.checking.pop_front().expect("verified chat");
        if self.checking.is_empty() {
            if self.story == ClusterStory::Restart && self.tasks.len() < CLUSTER_CHATS {
                self.members.stop_one("w1");
                self.stage = ClusterChatStage::StoppedOne;
            } else {
                self.members.stop();
                self.stage = ClusterChatStage::Stopping;
            }
        } else {
            self.poll(client);
        }
    }
    fn replied(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        if self.story == ClusterStory::Hung {
            if read.worker() == Some("w1") && read.waiting() {
                match self.tasks.len() {
                    1 => {
                        self.healed_attempt = Some(read.attempt());
                        self.linked_before = self.members.linked("w1");
                        self.members.signal("w1", Signal::Stop);
                        self.stage = ClusterChatStage::HealSilence;
                    }
                    2 => {
                        self.lost_attempt = Some(read.attempt());
                        self.members.up(&["w2"]);
                        self.stage = ClusterChatStage::FenceReady;
                    }
                    3 => {
                        self.checking.push_back(self.tasks[1].clone());
                        self.stage = ClusterChatStage::FenceCheck;
                    }
                    _ => unreachable!("heal, fence and new chat"),
                }
            }
            self.poll(client);
            return;
        }
        let first = self.tasks.len() < CLUSTER_CHATS;
        if let Some(worker) = read.worker().filter(|_| read.waiting()) {
            eprintln!(
                "client placement: task {} attempt {} worker {worker}",
                self.tasks.last().expect("chat"),
                read.attempt()
            );
            if self.story == ClusterStory::Each {
                assert!(self.workers.insert(worker.into()), "each chat on its own worker");
            } else {
                assert_eq!(worker, if first || self.story == ClusterStory::Restart { "w1" } else { "w2" });
            }
            if self.story == ClusterStory::Unexpected {
                self.queue(
                    client,
                    vec![
                        support::ctl::Call::new(&["task", "show", self.current(), "--until", "ended"]),
                        support::ctl::Call::new(&["unknown"]),
                    ],
                );
                self.members.signal("w1", Signal::Kill);
                self.stage = ClusterChatStage::Crashed;
            } else if self.story == ClusterStory::Kill {
                self.lost_attempt = Some(read.attempt());
                self.members.up(&["w2"]);
                self.stage = ClusterChatStage::Armed;
                self.poll(client);
            } else if first {
                if self.story == ClusterStory::Restart {
                    self.close(client, true);
                } else {
                    self.create_second(client);
                }
            } else {
                if self.story == ClusterStory::Each {
                    assert_eq!(self.workers.iter().map(String::as_str).collect::<Vec<_>>(), ["w1", "w2"]);
                }
                self.close(client, false);
            }
        } else {
            self.poll(client);
        }
    }
    fn hung_read(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        match self.stage {
            ClusterChatStage::HealSilence => {
                if read.hosts().iter().any(|host| host["name"] == "w1" && host["connected"] == false) {
                    self.members.signal("w1", Signal::Continue);
                    self.stage = ClusterChatStage::HealRelink;
                }
            }
            ClusterChatStage::HealRelink => {
                if self.members.linked("w1") > self.linked_before && read.connected("w1") {
                    assert_eq!(Some(read.attempt()), self.healed_attempt, "same attempt healed on w1");
                    self.close(client, true);
                    return;
                }
            }
            ClusterChatStage::FenceReady => {
                if self.members.is_up() && read.connected("w2") {
                    self.linked_before = self.members.linked("w1");
                    self.members.signal("w1", Signal::Stop);
                    self.stage = ClusterChatStage::FenceReplacement;
                }
            }
            ClusterChatStage::FenceReplacement => {
                if read.worker() == Some("w2") && read.waiting() && Some(read.attempt()) != self.lost_attempt {
                    self.stage = ClusterChatStage::FenceSnapshot;
                }
            }
            ClusterChatStage::FenceSnapshot => {
                self.fenced = Some(self.fenced_rows(read));
                self.members.signal("w1", Signal::Continue);
                self.stage = ClusterChatStage::FenceRelink;
            }
            ClusterChatStage::FenceRelink => {
                if self.members.linked("w1") > self.linked_before && read.connected("w1") {
                    self.create_second(client);
                    return;
                }
            }
            ClusterChatStage::FenceCheck => {
                // w1's hello listing b1 gets Cancel while b2 is
                // claimed (jig-domain-fleet attempt.rs:1030). Cancel ends b1's
                // run, completing its held reply turn; w1 sends that turn and
                // its terminal answer. The engine acknowledges the answer and
                // drops its payload (attempt.rs:539, :599), and drops any b1
                // turn (turn.rs:79). C waiting on w1's only slot proves w1
                // acted on Cancel: unchanged rows/turns show a drop, not silence.
                assert_eq!(Some(self.fenced_rows(read)), self.fenced, "fenced activity and committed turns unchanged");
                let old = self.lost_attempt.expect("fenced attempt");
                assert!(
                    read.rows.iter().any(|row| {
                        let end = &row["event"]["owner"]["source"]["line"]["run_ended"];
                        end["attempt"] == old && end["end"]["failed"]["class"] == "lost"
                    }),
                    "fenced attempt ended lost"
                );
                self.checking.clear();
                self.close(client, false);
                return;
            }
            ClusterChatStage::Start
            | ClusterChatStage::Created
            | ClusterChatStage::Replied
            | ClusterChatStage::WaitingSlot
            | ClusterChatStage::Crashed
            | ClusterChatStage::Armed
            | ClusterChatStage::Killed
            | ClusterChatStage::StoppedOne
            | ClusterChatStage::Relinking
            | ClusterChatStage::Closed
            | ClusterChatStage::Verifying
            | ClusterChatStage::Stopping => unreachable!("hung read stage"),
        }
        self.poll(client);
    }
    fn fenced_rows(&self, read: &ClusterRead) -> (Vec<serde_json::Value>, Vec<serde_json::Value>) {
        let old = self.lost_attempt.expect("fenced attempt");
        let activity = read
            .rows
            .iter()
            .filter(|row| {
                let line = &row["event"]["owner"]["source"]["line"];
                ["run_began", "run_ended", "turn"].iter().any(|kind| line[kind]["attempt"] == old)
            })
            .cloned()
            .collect();
        let turns = read
            .rows
            .iter()
            .filter(|row| row["event"]["owner"]["source"]["line"]["turn"].is_object())
            .cloned()
            .collect();
        (activity, turns)
    }
    fn loss(&self, read: &ClusterRead) {
        let old = self.lost_attempt.expect("killed attempt");
        let replacement = read.attempt();
        assert_ne!(replacement, old, "replacement attempt on w2");
        let lines: Vec<_> = read.rows.iter().map(|row| &row["event"]["owner"]["source"]["line"]).collect();
        assert_eq!(lines.iter().filter(|line| line["run_began"].is_object()).count(), LOST_AND_RETRY);
        for (attempt, end) in
            [(old, serde_json::json!({"failed": {"class": "lost"}})), (replacement, serde_json::json!("parked"))]
        {
            assert!(lines.iter().any(|line| line["run_began"]["attempt"] == attempt), "attempt began");
            let ended = lines.iter().find(|line| line["run_ended"]["attempt"] == attempt).expect("attempt ended");
            assert_eq!(ended["run_ended"]["end"], end);
        }
        assert!(!lines.iter().any(|line| line["turn"]["attempt"] == old), "killed reply was never committed");
    }
    fn close(&mut self, client: &mut support::cluster::Root, first_only: bool) {
        use support::ctl::Call;
        let mut close = Vec::new();
        for task in &self.tasks {
            if matches!(self.story, ClusterStory::Restart | ClusterStory::Hung)
                && !first_only
                && self.tasks.first() == Some(task)
            {
                continue;
            }
            self.checking.push_back(task.clone());
            close.push(Call::new(&["chat", "close", task, "--key", &format!("close-{task}")]));
            close.push(Call::new(&["task", "show", task, "--until", "ended"]));
        }
        self.queue(client, close);
        self.stage = ClusterChatStage::Closed;
    }
}

fn cluster_settings(file: &std::path::Path, reads: &[support::Ended]) {
    let mut fixture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(file).expect("cluster settings FILE")).expect("settings JSON");
    for read in reads {
        let current = support::ctl::json(read);
        let item = fixture["items"]
            .as_array_mut()
            .expect("settings items")
            .iter_mut()
            .find(|item| item["setting"] == current["setting"])
            .expect("cluster setting override");
        let overrides = item["set"].take();
        let mut value = current["value"].clone();
        value.as_object_mut().expect("whole setting").extend(
            overrides.as_object().expect("setting overrides").iter().map(|(key, value)| (key.clone(), value.clone())),
        );
        item["set"] = value;
    }
    support::write_private(file, &serde_json::to_vec(&fixture).expect("cluster settings JSON"));
}
struct ClusterRead {
    task: serde_json::Value,
    accounts: serde_json::Value,
    rows: Vec<serde_json::Value>,
    turns: bool,
}
impl ClusterRead {
    fn new(task: serde_json::Value, accounts: serde_json::Value) -> Self {
        Self { task, accounts, rows: Vec::new(), turns: false }
    }
    fn owner(&self) -> &serde_json::Value {
        &self.task["chip"]["details"]["owner"]
    }
    fn waiting(&self) -> bool {
        self.owner()["details"]["observation"]["work"] == "waiting"
    }
    fn attempt(&self) -> u64 {
        self.owner()["details"]["attempt"].as_u64().expect("chat attempt")
    }
    fn hosts(&self) -> &[serde_json::Value] {
        self.accounts["accounts"]["deployment_facts"]["program_hosts"].as_array().expect("deployment hosts")
    }
    fn connected(&self, name: &str) -> bool {
        self.hosts().iter().any(|host| host["name"] == name && host["connected"] == true)
    }
    fn worker(&self) -> Option<&str> {
        let channel = self.owner()["details"]["observation"]["channel"].as_u64()?;
        self.hosts().iter().find(|host| host["channel"] == channel && host["connected"] == true)?["name"].as_str()
    }
    fn replies(&self, text: &str) -> usize {
        self.rows
            .iter()
            .filter_map(|row| row["event"]["owner"]["decoded"]["turn"]["decoded"]["messages"].as_array())
            .flatten()
            .filter(|message| message["speaker"] == "assistant")
            .flat_map(|message| message["blocks"].as_array().expect("message blocks"))
            .filter_map(|block| block["text"]["text"].as_str())
            .map(|words| words.matches(text).count())
            .sum()
    }
}
fn complete_cluster_json(value: &serde_json::Value) {
    if let Some(cut) = value.get("cut") {
        assert_eq!(cut, false, "complete committed transcript");
    }
    if let Some(left) = value.get("bytes_left") {
        assert_eq!(left, 0, "complete turn text");
    }
    for item in
        value.as_object().into_iter().flat_map(serde_json::Map::values).chain(value.as_array().into_iter().flatten())
    {
        complete_cluster_json(item);
    }
}

impl skein_world::Referee<support::cluster::Root> for ClusterChatJudge {
    fn act(&mut self, now: skein_lib::Time, hosts: &mut [support::cluster::Root]) {
        use skein_shell::Host;
        use support::ctl::json;
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = ClusterChatStage::Stopping;
            return;
        }
        let (client, _) = hosts.split_last_mut().expect("cluster client root");
        if matches!(self.stage, ClusterChatStage::StoppedOne) && self.members.ended("w1") {
            self.members.restart("w1");
            self.stage = ClusterChatStage::Relinking;
            self.poll(client);
        }
        if self.members.is_up() || self.members.ended("w1") {
            if matches!(self.stage, ClusterChatStage::Start) {
                self.begin(client);
            } else if client.is_empty() && !matches!(self.stage, ClusterChatStage::Start | ClusterChatStage::Stopping) {
                let support::cluster::Root::Client { calls, .. } = client else { panic!("client root") };
                let calls = calls.take().expect("completed cluster calls");
                for ended in calls.ends() {
                    assert_eq!(ended.exit, Some(Exit::Code(0)), "ctl: {}", String::from_utf8_lossy(&ended.stderr));
                }
                match self.stage {
                    ClusterChatStage::Created => {
                        let created = json(calls.ends().last().expect("created chat"));
                        let task = created["outcome"]["done"]["document"]["created"]["first"]
                            .as_u64()
                            .unwrap_or_else(|| panic!("created chat number: {created}"))
                            .to_string();
                        self.tasks.push(task);
                        self.stage = if self.story == ClusterStory::Later && self.tasks.len() == CLUSTER_CHATS {
                            ClusterChatStage::WaitingSlot
                        } else {
                            ClusterChatStage::Replied
                        };
                        self.poll(client);
                    }
                    ClusterChatStage::Replied
                    | ClusterChatStage::WaitingSlot
                    | ClusterChatStage::Relinking
                    | ClusterChatStage::Verifying
                    | ClusterChatStage::Armed
                    | ClusterChatStage::Killed
                    | ClusterChatStage::HealSilence
                    | ClusterChatStage::HealRelink
                    | ClusterChatStage::FenceReady
                    | ClusterChatStage::FenceReplacement
                    | ClusterChatStage::FenceSnapshot
                    | ClusterChatStage::FenceRelink
                    | ClusterChatStage::FenceCheck => {
                        if let Some(read) = self.read(client, calls.ends()) {
                            self.advance_read(client, &read);
                        }
                    }
                    ClusterChatStage::Closed => {
                        self.stage = ClusterChatStage::Verifying;
                        self.poll(client);
                    }
                    ClusterChatStage::Start
                    | ClusterChatStage::Stopping
                    | ClusterChatStage::StoppedOne
                    | ClusterChatStage::Crashed => {
                        unreachable!("active calls")
                    }
                }
            }
        }
    }
    fn observe(&mut self, now: skein_lib::Time, hosts: &[support::cluster::Root]) {
        use skein_shell::Host;
        let (client, hosts) = hosts.split_last().expect("cluster client root");
        self.members.observe(now, hosts);
        if self.members.failed() {
            if self.story != ClusterStory::Unexpected {
                return;
            }
            let support::cluster::Root::Client { calls: Some(calls), .. } = client else {
                panic!("failed client's calls")
            };
            if self.story == ClusterStory::Unexpected && matches!(self.stage, ClusterChatStage::Crashed) {
                assert!(!calls.is_empty(), "in-flight child not yet reaped");
            }
            if self.story == ClusterStory::Unexpected && self.members.stopped().is_some() {
                assert!(calls.is_empty(), "client settled before remaining members stop");
                let [cancelled] = calls.ends() else { panic!("one in-flight call; queued unknown never started") };
                assert_eq!(cancelled.exit, Some(Exit::Signal(KILL_SIGNAL)), "client killed, waited and reaped");
            }
            return;
        }
        if let Some(stopped) = self.members.stopped() {
            let expected = if self.story == ClusterStory::Restart {
                vec![
                    ("w1", FIRST_LIFE),
                    ("w1", SECOND_LIFE),
                    ("engine", FIRST_LIFE),
                    ("llm", FIRST_LIFE),
                    ("forge", FIRST_LIFE),
                ]
            } else if self.story == ClusterStory::Kill {
                vec![
                    ("w1", FIRST_LIFE),
                    ("w2", FIRST_LIFE),
                    ("engine", FIRST_LIFE),
                    ("llm", FIRST_LIFE),
                    ("forge", FIRST_LIFE),
                ]
            } else {
                vec![
                    ("w2", FIRST_LIFE),
                    ("w1", FIRST_LIFE),
                    ("engine", FIRST_LIFE),
                    ("llm", FIRST_LIFE),
                    ("forge", FIRST_LIFE),
                ]
            };
            assert_eq!(stopped.iter().map(|s| (s.name.as_str(), s.life)).collect::<Vec<_>>(), expected);
            for member in stopped {
                if self.story == ClusterStory::Kill && member.name == "w1" {
                    assert_eq!(member.ending, support::cluster::Ending::Kill);
                    assert_eq!(member.exit, Some(Exit::Signal(KILL_SIGNAL)));
                    assert!(!member.killed && member.survivors.is_empty(), "intentional kill needed no cleanup");
                    continue;
                }
                let alone = self.story == ClusterStory::Restart && member.name == "w1" && member.life == FIRST_LIFE;
                assert_eq!(
                    member.ending,
                    if alone { support::cluster::Ending::Alone } else { support::cluster::Ending::Cluster }
                );
                clean_cluster_exit(member);
            }
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: skein_lib::Time) -> Option<String> {
        (now >= self.deadline && !self.members.passed()).then(|| {
            format!(
                "cluster {:?} waited in {:?}: {}; last read: {}",
                self.story,
                self.stage,
                match self.stage {
                    ClusterChatStage::Start => "requested members ready",
                    ClusterChatStage::Created => "chat creation",
                    ClusterChatStage::Replied => "waiting run and connected placement",
                    ClusterChatStage::WaitingSlot => "due second chat without attempt or observation",
                    ClusterChatStage::Crashed => "unexpected exit with a call in flight",
                    ClusterChatStage::Armed => "w2 linked and connected before the kill",
                    ClusterChatStage::Killed => "w1 tree settled and a new attempt waiting on w2",
                    ClusterChatStage::HealSilence => "heal: w1 program host connected:false after silence",
                    ClusterChatStage::HealRelink => "heal: another w1 linked line and connected host",
                    ClusterChatStage::FenceReady => "fence: w2 connected before stopping w1",
                    ClusterChatStage::FenceReplacement => "fence: different attempt waiting on w2 before continuing w1",
                    ClusterChatStage::FenceSnapshot =>
                        "fence: complete old activity and committed turns before continue",
                    ClusterChatStage::FenceRelink => "fence: another w1 linked line and connected host before chat C",
                    ClusterChatStage::FenceCheck =>
                        "fence: unchanged old activity and turns after chat C waiting on w1",
                    ClusterChatStage::Closed => "closed chats ended",
                    ClusterChatStage::Verifying => "ended done and complete committed turns",
                    ClusterChatStage::StoppedOne => "w1's first life settled",
                    ClusterChatStage::Relinking => "w1's second linked line and connected host",
                    ClusterChatStage::Stopping => "every process tree settled",
                },
                self.last_read
            )
        })
    }
    fn passed(&self) -> bool {
        matches!(self.stage, ClusterChatStage::Stopping) && self.members.passed()
    }
}

#[test]
fn a_default_box_with_a_sign_in_registration_starts_without_dns() {
    use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("first-run-no-dns");
    let hosts = scratch.path().join("hosts");
    let nsswitch = scratch.path().join("nsswitch.conf");
    fs::write(&hosts, b"").expect("empty names");
    fs::write(&nsswitch, b"hosts: files\npasswd: files\ngroup: files\n").expect("no DNS resolver");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    for path in &paths {
        fs::create_dir(path).expect("empty deployment directories");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private deployment directories");
    }
    let mut init = std::process::Command::new(skein_scratch::paths::binary("temper-box"));
    init.arg("init").arg("--box");
    for (flag, path) in ["--config", "--data", "--cache"].into_iter().zip(&paths) {
        init.arg(flag).arg(path);
    }
    let initialized = init.output().expect("initialize the default document without resolving names");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    support::own_listener_files(&paths[0]);
    // Smith also resolves the agent's endpoints at preflight. Supply local endpoint
    // facts there so this regression reaches accounts_runtime; forge defaults stay intact.
    let agent_path = paths[0].join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(&agent_path).expect("agent fixture")).expect("agent JSON");
    for endpoint in agent["endpoints"].as_array_mut().expect("endpoints") {
        if endpoint["address"].as_str().expect("endpoint address").parse::<std::net::SocketAddr>().is_err() {
            endpoint["address"] = "127.0.0.1:9".into();
        }
    }
    fs::write(agent_path, serde_json::to_vec(&agent).expect("local agent fixture")).expect("local preflight endpoints");
    // The default registration reaches the sign-in startup resolver.
    let document_path = paths[0].join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&document_path).expect("default document")).expect("JSON");
    document["application"]["limits"]["overrides"]["git_component.operations"] = 1.into();
    document["application"]["limits"]["overrides"]["checkout.workspaces"] = 1.into();
    document["workers"][0]["slots"] = 1.into();
    fs::write(document_path, serde_json::to_vec(&document).expect("sign-in fixture")).expect("registered sign-in");
    // The private network has only loopback. Files-only name lookup and a private /run
    // exclude DNS and host resolver daemons; the demo peers alone use its sockets.
    let script = r#"set -eu
/usr/bin/mount --make-rprivate /
/usr/bin/mount --bind "$1" /etc/hosts
/usr/bin/mount --bind "$2" /etc/nsswitch.conf
/usr/bin/mount -t tmpfs tmpfs /run
/usr/bin/ip link set lo up
shift 2
exec "$@"
"#;
    let mut arguments: Vec<OsString> = ["-Urnm", "/bin/sh", "-c", script, "no-dns"].map(OsString::from).into();
    arguments.extend([
        hosts.into_os_string(),
        nsswitch.into_os_string(),
        skein_scratch::paths::binary("temper-box").into(),
    ]);
    for (flag, path) in ["--config", "--data", "--cache"].into_iter().zip(&paths) {
        arguments.extend([flag.into(), path.as_os_str().into()]);
    }
    let ended = support::boxed::run_command(
        skein_world::end_to_end::Command {
            program: "/usr/bin/unshare".into(),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        None,
        support::boxed::Plan::Stop,
        BOX_DEADLINE,
    );
    let diagnostic = String::from_utf8_lossy(&ended.stderr).into_owned();
    let listening = support::boxed::serve_listening(&paths[1]);
    scratch.remove();
    assert!(!diagnostic.contains("application.forges"), "forge preparation refused: {diagnostic}");
    assert!(!diagnostic.contains("workers[0].agent"), "agent preflight fixture refused: {diagnostic}");
    assert!(
        listening || diagnostic.contains("failed to lookup address information"),
        "unexpected startup refusal: {diagnostic}"
    );
    assert!(listening, "no-DNS start must reach serve's listening: {diagnostic}");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{diagnostic}");
    assert_eq!(ended.left, Some(Vec::new()), "no process outlived the box: {diagnostic}");
}

// testing/process-tier.md: the person's App, one push and both services restarted.

// Shows the opener and timer cancellation on three processes. Leaves out pools,
// lending, host moves and the other omissions in testing/reference-case.md's 4.1/4.3 rows;
// A and the reviewer leave no placement evidence in an ended document or workspace.

// Leaves out the director as asker, plan pacing, B's model, repeated questions,
// and amending only A's model (testing/reference-case.md's 4.4 row).
// On ws alone, no s2: a worker's own sources and sign-in are undesigned.
// Past 6.1's 2 s guide, as its real-loop twin is, by its seven activations.

// Leaves out s2's sign-in, credentials never copied between machines, B's work,
// five hours and a session that never ends (testing/reference-case.md's 4.5 row).
// On ws alone, no s2: a worker's own sources and sign-in are undesigned.

// Leaves out s2's sign-in, credentials never copied between machines, B's work,
// five hours and a session that never ends (testing/reference-case.md's 4.5 row).
// On ws alone, no s2: a worker's own sources and sign-in are undesigned.
// Past 6.1's 2 s guide, as its real-loop twin is, by its reset: one second,
// the shortest a provider's wire carries.

// Shows the retained question and one answer across ws's two lives. Leaves out
// the hang past grace, three questions and the other limits of testing/reference-case.md's 4.6 row.
// The retry runs on s1 or s2 by that row's act order: ws stays ended and unlinked
// until after the ended chat's watch joins; the oracle checks both attempts.

// testing/process-tier.md: the chat's connected GitHub change survives both services restarting.

// Shows the retained question and one answer across ws's two lives. Leaves out
// the hang past grace, three questions and the other limits of testing/reference-case.md's 4.6 row.
// The retry runs on s1 or s2 by that row's act order: ws stays ended and unlinked
// until after the ended chat's watch joins; the oracle checks both attempts.
#[test]
#[ignore = "4.6 process fixture expects engine.llm.models in startup configuration after models moved to settings (temper/docs/design/testing/reference-case.md; temper/docs/design/protocol/configuration/README.md, section 2 and temper/docs/design/protocol/configuration/stock.md)"]
fn a_restarted_workstation_keeps_the_directors_question_and_answers_it_once_as_processes_without_a_hang_past_the_grace()
{
    use temper_real_loop_world::case::{self, acts, oracle};
    let run = support::case::run(
        &["ws", "s1", "s2"],
        case::fixture_4_6(),
        case::scripts::step_4_6,
        case::scripts::step_4_6_rules,
        acts::step_4_6(0),
        &[case::HANG],
    );
    for (at, call) in run.calls().iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(skein_io::kernel::Exit::Code(0)),
            "call {at}: {}",
            String::from_utf8_lossy(&call.stderr)
        );
    }
    oracle::assert_met(&oracle::step_4_6(&run.observed(acts::READS_4_6)));
    assert_eq!(run.first_placed().get(&case::HANG).map(String::as_str), Some("ws"));
}

// On Linux its restored Forgejo starts while discovery answers 502 with the
// engine held down, then signs in after it returns, without a Forgejo restart.
#[test]
fn a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click() {
    let scratch = support::Scratch::new("forgejo-box-sign-in");
    support::forgejo_box_sign_in::run(scratch.path());
    scratch.remove();
}

#[test]
fn an_owned_forgejo_accepts_a_public_source_once_and_exchanges_its_code() {
    let scratch = support::Scratch::new("forgejo_public_source");
    support::forgejo_sign_in::public(scratch.path());
    scratch.remove();
}
#[test]
fn an_owned_forgejo_signs_its_member_in_through_temper() {
    let scratch = support::Scratch::new("forgejo_sign_in");
    support::forgejo_sign_in::run(scratch.path());
    scratch.remove();
}

// The capture witness uses Linux's abstract Unix sockets.
#[cfg(target_os = "linux")]
#[test]
fn a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart()
 {
    support::forgejo_elsewhere::restart();
}

#[cfg(target_os = "linux")]
#[test]
fn a_story_box_and_its_steps_die_when_the_test_process_is_killed() {
    use std::{
        fs,
        process::{Command, Stdio},
    };
    use support::parent::{Guard, wait};
    if let Some(root) = temper_test_env::value(&temper_test_env::BOX_PARENT_TEST) {
        let root = std::path::Path::new(&root);
        let command = support::boxed::dying_with_test(skein_world::end_to_end::Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: support::configuration::flags(root),
            directory: root.into(),
            environment: Vec::new(),
        });
        // The same test thread spawns and waits: PR_SET_PDEATHSIG follows it.
        let mut box_process = Command::new(command.program)
            .args(command.arguments)
            .current_dir(command.directory)
            .env_clear()
            .envs(command.environment)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("wrapped box");
        fs::write(root.join("box.pid.tmp"), box_process.id().to_string()).expect("box pid");
        fs::rename(root.join("box.pid.tmp"), root.join("box.pid")).expect("publish box pid");
        let _ended = box_process.wait().expect("box wait");
        return;
    }
    let scratch = support::Scratch::new("box-dies-with-test");
    // Init before box.pid exists: a first run requires an empty configuration directory.
    let output = Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .arg("--box")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    support::own_listener_files(scratch.path());
    let child = Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "a_story_box_and_its_steps_die_when_the_test_process_is_killed", "--nocapture"])
        .env(temper_test_env::BOX_PARENT_TEST.name, scratch.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("child test");
    let mut guard = Guard::new(child);
    let pid_file = scratch.path().join("box.pid");
    wait(&mut guard, |_| pid_file.exists());
    let box_pid = fs::read_to_string(pid_file).expect("box pid").parse().expect("pid number");
    guard.record(box_pid, guard.child.as_ref().expect("child test").id());
    wait(&mut guard, |guard| {
        guard.descendants(box_pid);
        guard.has_descendants()
    });
    guard.child.as_mut().expect("child test").kill().expect("SIGKILL test alone");
    guard.child.as_mut().expect("child test").wait().expect("reap test");
    guard.wait_all();
    scratch.remove();
}

fn ctl_assert_success(calls: &[support::Ended]) {
    for (index, call) in calls.iter().enumerate() {
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {index}: {}", String::from_utf8_lossy(&call.stderr));
        assert!(call.stderr.is_empty(), "call {index}: {}", String::from_utf8_lossy(&call.stderr));
    }
}

#[test]
fn playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::{
        ctl::{Call, Step},
        first_repository,
    };
    let scratch = support::Scratch::new("forgejo-playground-copy");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    for path in &paths {
        fs::create_dir(path).expect("empty first-run directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private directory");
    }
    let flags = support::boxed::echo_steps(deployment_flags(&paths));
    let _cached = cache_forgejo_binaries(&paths[2]);
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["forge", "setup", "--playground"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
    ];
    steps.extend(first_repository::playground_read_steps(scratch.path()));
    steps.push(Step::Run(Call::new(&["project", "show", "1", "--json"])));
    let ended = support::boxed::run_ctl_with_llm(
        scratch.path(),
        flags,
        scratch.path(),
        steps,
        BOX_DEADLINE,
        |_| first_repository::playground_read_evidence(&paths[0], scratch.path()),
        first_repository::playground_read_llm(&paths[0]),
    );
    let stderr = String::from_utf8_lossy(&ended.stderr);
    assert_eq!(ended.exit, Some(Exit::Code(0)), "box drained: {stderr}");
    assert_eq!(ended.left, Some(Vec::new()), "no processes outlive the box");
    assert_eq!(ended.calls.len(), 9, "setup, reads and pull completed: {stderr}");
    for call in &ended.calls {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "stdout: {}; stderr: {}; box: {stderr}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    assert_playground_setup(&ended);
    scratch.remove();
}

fn assert_playground_setup(ended: &support::boxed::CtlOutcome<support::first_repository::PlaygroundReads>) {
    let adoption = String::from_utf8_lossy(&ended.calls[1].stdout);
    assert!(adoption.lines().any(|line| line == "project: 1 · widget"), "{adoption}");
    assert!(adoption.contains("Adopted playground/widget on forge 1 as owned."), "{adoption}");
    assert!(adoption.contains("It is the project's home."), "{adoption}");
    let rows = support::ctl::json(&ended.calls[2]);
    let row = rows["forges"].as_array().expect("forge rows").iter().find(|row| row["number"] == 1).expect("forge 1");
    assert_eq!(row["first"]["state"], "made");
    assert_eq!(row["state"], "connected");
    assert_eq!(row["first"]["org"], "playground");
    assert_eq!(row["first"]["repository"], "widget");
    assert_eq!(row["first"]["playground"], true);
    let evidence = ended.checked.as_ref().expect("live worker reads");
    assert_eq!(evidence.panel.exit, Some(Exit::Code(0)), "pull panel read succeeded");
    assert!(evidence.producer.contains("\"path\":\".forgejo/workflows/ci.yml\"}\nRead "), "{}", evidence.producer);
    assert!(evidence.producer.contains("\"path\":\"src/main.rs\"}\nRead "), "{}", evidence.producer);
    // ctl prints read counts; the service test compares every source byte.
    assert!(evidence.producer.contains("Read 7 of 7 lines; skipped 0"), "{}", evidence.producer);
    let panel = support::ctl::json(&evidence.panel);
    assert_eq!(panel["landing"], "main", "{panel}");
    assert_eq!(panel["pull"]["address"], format!("http://{}/playground/widget/pulls/1", evidence.address));
    let project = support::ctl::json(&ended.calls[8]);
    assert_eq!(project["number"], 1);
    assert_eq!(project["name"]["bytes"], "widget");
    let resources = project["resources"].as_array().expect("project repositories");
    assert_eq!(resources.len(), 1);
    assert_eq!(
        resources[0]["part"],
        serde_json::json!({
            "connector":1, "kind":"resource", "key":{"path":["0001", "00000001"]}
        })
    );
    assert_eq!(resources[0]["role"], "owned");
    assert_eq!(resources[0]["home"], true);
}

// The move uses a fake issuer and no forge or model work.
fn restart_deployment(paths: &[std::path::PathBuf; 3]) -> skein_fake_peers::oauth::Peer {
    support::forge_land::fake_deployment(paths)
}

#[test]
fn a_restart_over_an_imported_then_exported_account_answers_ctl() {
    account_transfer_restart("restart-imported-exported-account");
}

/// temper/docs/design/testing/live-tier.md: the launcher's return after a sitting stopped.
#[test]
fn a_held_record_returns_on_a_box_of_its_own_after_the_storys_box_stopped() {
    use std::os::unix::fs::PermissionsExt;
    use support::ctl::{Call, Step};
    let scratch = support::Scratch::new("held-login-return-box");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    imported_restart_account(scratch.path(), &paths);
    let held = scratch.path().join("incoming-account.move");
    let temper = skein_scratch::paths::binary("temper-box");
    let sitting = support::boxed::run_ctl_with_account(
        scratch.path(),
        support::boxed::echo_steps(deployment_flags(&paths)),
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["account", "import", held.to_str().expect("move file"), "--key", "guard"])),
            Step::Run(Call::new(&["account", "list", "--until", "usable"])),
        ],
        BOX_DEADLINE,
        || (),
        support::forge_land::fake_face(support::sign_in::restart_issued_issuer(&paths[0]), no_model_scripts()),
    );
    ctl_assert_success(&sitting.calls);
    assert_eq!(sitting.calls.len(), 3, "all import acts answered");
    assert_eq!(sitting.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&sitting.stderr));
    assert_eq!(sitting.left, Some(Vec::new()), "the sitting drained");
    assert!(!held.exists(), "the record moved in");
    support::forge_land::rung::return_on_a_box_of_its_own(&temper, scratch.path(), &paths, &held)
        .expect("return on its own box");
    let metadata = std::fs::symlink_metadata(&held).expect("the return box exported the record");
    assert!(metadata.is_file() && metadata.len() > 0);
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(paths[0].join("temper.json")).expect("configuration")).expect("JSON");
    let listener = config["application"]["clients"]["listener"]["socket"].as_str().expect("client listener");
    assert!(std::net::TcpStream::connect(listener).is_err(), "the return's box stopped");
    scratch.remove();
}

#[test]
fn a_restart_with_another_box_sharing_its_cache_answers_ctl() {
    support::restart::shared_cache_restart("restart-shared-cache");
}

fn imported_restart_account(scratch: &std::path::Path, paths: &[std::path::PathBuf; 3]) {
    use support::ctl::{Call, Step};
    let donor = scratch.join("donor");
    std::fs::create_dir(&donor).expect("fixture donor deployment");
    let donor_paths = ["config", "data", "cache"].map(|name| donor.join(name));
    let issuer = restart_deployment(&donor_paths);
    let file = scratch.join("incoming-account.move");
    let steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["account", "connect", "1", "--key", "donor-account"])),
        Step::Run(Call::new(&["account", "list", "--until", "usable"])),
        Step::Run(Call::new(&["account", "export", "1", file.to_str().expect("fixture move path")])),
    ];
    let donor = support::boxed::run_ctl_with_account(
        &donor,
        support::boxed::echo_steps(deployment_flags(&donor_paths)),
        &donor,
        steps,
        BOX_DEADLINE,
        || (),
        support::forge_land::fake_face(issuer, no_model_scripts()),
    );
    ctl_assert_success(&donor.calls);
    assert!(donor.checked.is_some(), "donor exported its fixture account: {}", String::from_utf8_lossy(&donor.stderr));
    assert_eq!(donor.exit, Some(Exit::Code(0)));
    assert_eq!(donor.left, Some(Vec::new()), "the donor tree drained");
    // Draw the recipient's listeners after the donor's connections have drained.
    drop(restart_deployment(paths));
    // A move must find the very same provider registration at its destination.
    let read = |path| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(path).expect("fixture configuration")).expect("configuration JSON")
    };
    let donor = read(donor_paths[0].join("temper.json"));
    let config = paths[0].join("temper.json");
    let mut destination = read(config.clone());
    destination["engine"]["llm"]["providers"][0]["sources"]["sign_in"] =
        donor["engine"]["llm"]["providers"][0]["sources"]["sign_in"].clone();
    support::write_private(&config, &serde_json::to_vec(&destination).expect("fixture configuration JSON"));
}

fn account_transfer_restart(case: &str) {
    use support::ctl::{Call, Step};
    let scratch = support::Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    imported_restart_account(scratch.path(), &paths);
    let flags = support::boxed::echo_steps(deployment_flags(&paths));
    let file = scratch.path().join("incoming-account.move");
    let steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["account", "import", file.to_str().expect("move file"), "--key", "restart-account"])),
        Step::Run(Call::new(&["account", "list", "--until", "usable"])),
    ];
    let first = support::boxed::run_ctl_with_account(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        steps,
        BOX_DEADLINE,
        || {
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(paths[0].join("temper.json")).expect("configuration"))
                    .expect("JSON");
            let upstream = value["application"]["clients"]["listener"]["socket"]
                .as_str()
                .expect("client address")
                .parse()
                .expect("socket");
            let moving = support::account_transfer::export_pending(
                scratch.path(),
                upstream,
                1,
                &scratch.path().join("exported-login"),
            );
            assert_ne!((moving["accounts"]["party"].as_array().expect("moving account")).len(), 0);
        },
        support::forge_land::fake_face(support::sign_in::restart_issued_issuer(&paths[0]), no_model_scripts()),
    );
    assert_eq!(first.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.left, Some(Vec::new()), "the interrupted export's box drained");
    ctl_assert_success(&first.calls);
    assert!(first.checked.is_some());
    assert!(!file.exists(), "the account was imported");
    let second = support::boxed::run_ctl_with_account(
        scratch.path(),
        flags,
        scratch.path(),
        vec![
            Step::Run(Call::new(&["account", "list", "--until", "usable"])),
            Step::Run(Call::new(&["account", "list", "--json"])),
        ],
        BOX_DEADLINE,
        || (),
        support::forge_land::fake_face(support::sign_in::restart_issued_issuer(&paths[0]), no_model_scripts()),
    );
    assert_eq!(second.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(second.left, Some(Vec::new()), "the restarted box drained");
    ctl_assert_success(&second.calls);
    assert!(second.checked.is_some());
    let accounts = support::ctl::json(second.calls.last().expect("accounts read"));
    assert_eq!(accounts["accounts"]["party"].as_array().expect("party accounts").len(), 1);
    scratch.remove();
}

fn no_model_scripts() -> skein_fake_llm_domain::api::Scripts {
    skein_fake_llm_domain::api::Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([]) }
}

// The retained process identities and the stopped-parent observation are Linux's.
#[cfg(target_os = "linux")]
#[test]
fn a_killed_serve_keeps_its_port_and_prints_a_fresh_link() {
    use std::{fs, net::TcpStream};
    let mut boxed = box_link::Boxed::start("killed-serve-link", None);
    let first = boxed.link();
    let address = box_link::address(&first);
    boxed.wait(|boxed| support::boxed::serve_listening(&boxed.root().join("data")));
    let parent = boxed.guard.child.as_ref().expect("box child").id();
    let mut processes = support::parent::Guard::observing_test();
    let engine = processes
        .pids()
        .into_iter()
        .find(|pid| {
            fs::read_link(format!("/proc/{pid}/exe")).ok().as_deref()
                == Some(skein_scratch::paths::binary("temper-box").as_path())
                && fs::read(format!("/proc/{pid}/cmdline"))
                    .is_ok_and(|bytes| bytes.split(|byte| *byte == 0).any(|word| word == b"serve"))
        })
        .expect("the box's live serve process");
    // Keep the down interval observable without changing the configured backoff.
    support::parent::signal(boxed.guard.child.as_mut().expect("box child"), "STOP");
    boxed.wait(|_| {
        fs::read_to_string(format!("/proc/{parent}/status")).expect("owned box status").contains("State:\tT")
    });
    processes.signal(engine, Signal::Kill);
    processes.wait_for(engine);
    let connected =
        TcpStream::connect(address).expect("the held socket accepts while serve is dead and box is stopped");
    drop(connected);
    support::parent::signal(boxed.guard.child.as_mut().expect("box child"), "CONT");
    boxed.wait(|boxed| {
        fs::read_to_string(boxed.root().join("data/box/logs/serve.log"))
            .expect("serve log")
            .matches("listening clients ")
            .count()
            == 2
    });
    // Check replay before any sign-in spends the restarted serve's first secret.
    assert!(
        !box_link::signs_in(boxed.root(), "old-link-client", &first),
        "the block's first link is refused after restart"
    );
    let fresh = boxed.link();
    assert_eq!(box_link::address(&fresh), address, "the restart keeps the clients port");
    assert!(fresh != first, "the restart never replays the block's first link");
    let stderr = boxed.stderr();
    let block = [box_link::PRELUDE, box_link::footer(&first).as_slice()].concat();
    let after_block =
        stderr.strip_prefix(block.as_slice()).expect("the first sign-in block precedes restart diagnostics");
    assert!(after_block.ends_with(b"\n\n"), "a fresh link has its own blank line on stderr");
    assert!(!after_block.windows(box_link::PRELUDE.len()).any(|bytes| bytes == box_link::PRELUDE), "no second block");
    assert!(box_link::signs_in(boxed.root(), "restart-client", &fresh), "the restart's fresh link signs in as owner");
    let (stdout, _, _) = boxed.stop();
    let expected = [&first[..], b"\n", &fresh[..], b"\n"].concat();
    assert!(stdout == expected, "stdout holds only whole, unprefixed and untimed links");
}
